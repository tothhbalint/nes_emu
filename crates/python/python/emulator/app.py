import sys

from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QApplication,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QMainWindow,
    QPlainTextEdit,
    QPushButton,
    QSizePolicy,
    QSpinBox,
    QSplitter,
    QVBoxLayout,
    QWidget,
)


class Backend:
    """Thin adapter over the current Rust bridge surface.

    Exposed by ``PyNes`` today: ``load``, ``step`` (one instruction),
    ``snap``, ``disassemble_forward``, and ``run_until_breakpoints``.
    """

    def __init__(self) -> None:
        from emulator import Nes

        self.nes = Nes()
        self.loaded = False

    def load(self, path: str) -> None:
        self.loaded = False
        self.nes.load(path)
        self.loaded = True

    def snap(self) -> dict | None:
        if not self.loaded:
            return None
        s = self.nes.snap()
        return {
            "a": s.a,
            "x": s.x,
            "y": s.y,
            "pc": s.pc,
            "sp": s.sp,
            "status": s.status,
        }

    def disassemble_forward(self, start: int, count: int):
        if not self.loaded:
            return []
        return self.nes.disassemble_forward(start, count)

    def step_instruction(self) -> None:
        if not self.loaded:
            return
        self.nes.step()

    def run_until_breakpoints(self, breakpoints) -> int | None:
        if not self.loaded:
            return None
        return self.nes.run_until_breakpoints(sorted(breakpoints))


class DisasmView(QPlainTextEdit):
    """Read-only disassembly; double-clicking a line toggles a breakpoint."""

    line_double_clicked = Signal(int)

    def mouseDoubleClickEvent(self, event):
        cursor = self.cursorForPosition(event.position().toPoint())
        self.line_double_clicked.emit(cursor.blockNumber())
        event.accept()


FLAG_ORDER = [
    ("N", 7),
    ("V", 6),
    ("U", 5),
    ("B", 4),
    ("D", 3),
    ("I", 2),
    ("Z", 1),
    ("C", 0),
]


class MainWindow(QMainWindow):
    DISASM_LINES = 128

    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("NES CPU Test")
        self.setMinimumSize(720, 480)
        self.backend = Backend()
        self.running = False
        self.breakpoints: set[int] = set()
        self.disasm_entries = []

        mono = QFont("Menlo")
        mono.setStyleHint(QFont.StyleHint.Monospace)

        central = QWidget()
        root = QVBoxLayout(central)
        root.setContentsMargins(8, 8, 8, 8)
        root.setSpacing(8)

        root.addLayout(self._build_controls())
        root.addWidget(self._build_main_splitter(mono), 1)

        self.setCentralWidget(central)
        self.resize(1100, 700)

        self.timer = QTimer(self)
        self.timer.setInterval(16)
        self.timer.timeout.connect(self.on_tick)

        self.refresh()

    # --- layout ---
    def _build_controls(self) -> QHBoxLayout:
        self.load_btn = QPushButton("Load ROM…")
        self.step_btn = QPushButton("Step")
        self.run_btn = QPushButton("Run")
        self.bp_addr = QLineEdit()
        self.bp_addr.setPlaceholderText("addr")
        self.bp_addr.setMaximumWidth(80)
        self.add_bp_btn = QPushButton("Add")
        self.bp_at_pc_btn = QPushButton("At PC")
        self.clear_bps_btn = QPushButton("Clear breakpoints")
        self.status = QLabel("idle")
        self.step_budget = QSpinBox()
        self.step_budget.setRange(1, 1_000_000)
        self.step_budget.setValue(2000)
        self.step_budget.setToolTip("Instructions executed per UI tick while running")
        self.bp_addr.setToolTip("Hexadecimal address, e.g. C6BD")
        self.bp_at_pc_btn.setToolTip("Set a breakpoint at the current PC")

        self.step_btn.setEnabled(False)
        self.run_btn.setEnabled(False)
        self.bp_addr.setEnabled(False)
        self.add_bp_btn.setEnabled(False)
        self.bp_at_pc_btn.setEnabled(False)
        self.clear_bps_btn.setEnabled(False)

        self.load_btn.clicked.connect(self.on_load)
        self.step_btn.clicked.connect(self.on_step)
        self.run_btn.clicked.connect(self.toggle_run)
        self.bp_addr.returnPressed.connect(self.on_add_breakpoint)
        self.add_bp_btn.clicked.connect(self.on_add_breakpoint)
        self.bp_at_pc_btn.clicked.connect(self.on_break_at_pc)
        self.clear_bps_btn.clicked.connect(self.on_clear_breakpoints)

        controls = QHBoxLayout()
        controls.setSpacing(8)
        controls.addWidget(self.load_btn)
        controls.addWidget(self.step_btn)
        controls.addWidget(self.run_btn)
        controls.addWidget(QLabel("instrs/tick"))
        controls.addWidget(self.step_budget)
        controls.addSpacing(16)
        controls.addWidget(QLabel("bp"))
        controls.addWidget(self.bp_addr)
        controls.addWidget(self.add_bp_btn)
        controls.addWidget(self.bp_at_pc_btn)
        controls.addWidget(self.clear_bps_btn)
        controls.addWidget(self.status, 1)
        return controls

    def _build_main_splitter(self, mono: QFont) -> QSplitter:
        # Left: CPU panel (grows with the window).
        self.reg_labels: dict[str, QLabel] = {}
        reg_box = QGroupBox("CPU")
        form = QFormLayout(reg_box)
        form.setFieldGrowthPolicy(
            QFormLayout.FieldGrowthPolicy.AllNonFixedFieldsGrow
        )
        form.setLabelAlignment(Qt.AlignmentFlag.AlignRight)
        form.setHorizontalSpacing(16)

        for name in ("A", "X", "Y", "SP", "PC", "CYC", "FLAGS"):
            lbl = QLabel("--")
            lbl.setFont(mono)
            lbl.setMinimumWidth(60)
            lbl.setSizePolicy(
                QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred
            )
            self.reg_labels[name] = lbl
            form.addRow(name, lbl)

        cpu_panel = QWidget()
        cpu_layout = QVBoxLayout(cpu_panel)
        cpu_layout.setContentsMargins(0, 0, 0, 0)
        cpu_layout.addWidget(reg_box)
        cpu_layout.addStretch(1)

        # Right: disassembly over memory, resizable vertically.
        self.disasm = DisasmView()
        self.disasm.setReadOnly(True)
        self.disasm.setFont(mono)
        self.disasm.setPlaceholderText(
            "disassembly from PC — double-click a line to toggle a breakpoint"
        )
        self.disasm.line_double_clicked.connect(self.on_disasm_double_click)

        self.mem = QPlainTextEdit()
        self.mem.setReadOnly(True)
        self.mem.setFont(mono)
        self.mem.setPlainText("(memory API not exposed yet)")

        right_split = QSplitter(Qt.Orientation.Vertical)
        right_split.setChildrenCollapsible(False)
        right_split.addWidget(self.disasm)
        right_split.addWidget(self.mem)
        right_split.setStretchFactor(0, 3)
        right_split.setStretchFactor(1, 2)
        right_split.setSizes([420, 280])

        # Main: CPU panel | right panel, resizable horizontally.
        main_split = QSplitter(Qt.Orientation.Horizontal)
        main_split.setChildrenCollapsible(False)
        main_split.addWidget(cpu_panel)
        main_split.addWidget(right_split)
        main_split.setStretchFactor(0, 1)
        main_split.setStretchFactor(1, 3)
        main_split.setSizes([280, 820])
        return main_split

    # --- handlers ---
    def pause(self, message: str = "paused") -> None:
        self.running = False
        self.timer.stop()
        self.run_btn.setText("Run")
        self.status.setText(message)

    def on_load(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self, "Open ROM", "", "NES ROM (*.nes)"
        )
        if not path:
            return

        try:
            self.backend.load(path)
        except Exception as exc:  # noqa: BLE001 - surface load errors in the view
            self.pause("load failed")
            self.step_btn.setEnabled(False)
            self.run_btn.setEnabled(False)
            self.bp_addr.setEnabled(False)
            self.add_bp_btn.setEnabled(False)
            self.bp_at_pc_btn.setEnabled(False)
            self.clear_bps_btn.setEnabled(False)
            self.disasm.setPlainText(f"failed to load ROM: {exc}")
            return

        self.breakpoints.clear()
        self.pause("loaded")
        self.step_btn.setEnabled(True)
        self.run_btn.setEnabled(True)
        self.bp_addr.setEnabled(True)
        self.add_bp_btn.setEnabled(True)
        self.bp_at_pc_btn.setEnabled(True)
        self.clear_bps_btn.setEnabled(True)
        self.refresh()

    def on_step(self) -> None:
        if not self.backend.loaded:
            return
        self.pause("stepped")
        self.backend.step_instruction()
        self.refresh()

    def toggle_run(self) -> None:
        if not self.backend.loaded:
            return
        if self.running:
            self.pause("paused")
            return

        self.running = True
        self.run_btn.setText("Pause")
        self.status.setText("running")
        self.refresh()
        self.timer.start()

    def on_tick(self) -> None:
        if not self.backend.loaded:
            self.pause("not loaded")
            return

        # With breakpoints armed, the bridge runs synchronously until one is hit.
        if self.breakpoints:
            hit = self.backend.run_until_breakpoints(self.breakpoints)
            self.refresh()
            if hit is not None:
                self.pause(f"breakpoint at ${hit:04X}")
            return

        for _ in range(self.step_budget.value()):
            self.backend.step_instruction()
        self.refresh()

    def on_disasm_double_click(self, line: int) -> None:
        if line < 0 or line >= len(self.disasm_entries):
            return
        self.toggle_breakpoint(self.disasm_entries[line].pc)

    def on_add_breakpoint(self) -> None:
        raw = self.bp_addr.text().strip().lower()
        for prefix in ("$", "0x"):
            if raw.startswith(prefix):
                raw = raw[len(prefix):]
        if not raw:
            return

        try:
            addr = int(raw, 16)
        except ValueError:
            self.status.setText(f"not a hex address: {self.bp_addr.text()}")
            return
        if not 0 <= addr <= 0xFFFF:
            self.status.setText("address out of range (0000-FFFF)")
            return

        self.bp_addr.clear()
        self.toggle_breakpoint(addr, force_add=True)

    def on_break_at_pc(self) -> None:
        s = self.backend.snap()
        if s is None:
            return
        self.toggle_breakpoint(s["pc"], force_add=True)

    def toggle_breakpoint(self, addr: int, force_add: bool = False) -> None:
        if addr in self.breakpoints and not force_add:
            self.breakpoints.remove(addr)
            self.status.setText(f"breakpoint removed at ${addr:04X}")
        else:
            self.breakpoints.add(addr)
            self.status.setText(f"breakpoint added at ${addr:04X}")
        self.refresh()

    def on_clear_breakpoints(self) -> None:
        self.breakpoints.clear()
        self.refresh()

    # --- views ---
    def refresh(self) -> None:
        self.clear_bps_btn.setText(
            f"Clear breakpoints ({len(self.breakpoints)})"
            if self.breakpoints
            else "Clear breakpoints"
        )

        if not self.backend.loaded:
            self.clear_views()
            return

        s = self.backend.snap()
        if s is None:
            self.clear_views()
            return

        self.reg_labels["A"].setText(f"{s['a']:02X}")
        self.reg_labels["X"].setText(f"{s['x']:02X}")
        self.reg_labels["Y"].setText(f"{s['y']:02X}")
        self.reg_labels["SP"].setText(f"{s['sp']:02X}")
        self.reg_labels["PC"].setText(f"{s['pc']:04X}")
        self.reg_labels["CYC"].setText("--")  # not exposed by the bridge

        flags = "".join(
            name if s["status"] & (1 << bit) else "-"
            for name, bit in FLAG_ORDER
        )
        self.reg_labels["FLAGS"].setText(flags)

        self.update_disasm(s["pc"])

    def clear_views(self) -> None:
        for lbl in self.reg_labels.values():
            lbl.setText("--")
        self.disasm_entries = []
        self.disasm.setPlainText("(load a ROM to disassemble)")
        self.mem.setPlainText("(load a ROM)")

    def update_disasm(self, pc: int) -> None:
        if not self.backend.loaded:
            self.disasm_entries = []
            return

        entries = self.backend.disassemble_forward(pc, self.DISASM_LINES)
        self.disasm_entries = list(entries)

        rendered = []
        for e in entries:
            here = ">" if e.pc == pc else " "
            bp = "●" if e.pc in self.breakpoints else " "
            hexes = " ".join(f"{b:02X}" for b in e.opcode)
            rendered.append(f"{bp}{here} {e.pc:04X}  {hexes:<9} {e.mnemonic}")

        self.disasm.setPlainText("\n".join(rendered) or "(no disassembly)")


def main() -> None:
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    raise SystemExit(app.exec())


if __name__ == "__main__":
    main()
