import sys

from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QApplication,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
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

    Exposed by ``PyNes`` today: ``load``, ``step`` (one CPU clock cycle),
    ``snap``, and ``run_until_breakpoint``.
    """

    def __init__(self) -> None:
        from emulator import Nes

        self.nes = Nes()

    def load(self, path: str) -> None:
        self.nes.load(path)

    def snap(self) -> dict:
        s = self.nes.snap()
        return {
            "a": s.a,
            "x": s.x,
            "y": s.y,
            "pc": s.pc,
            "sp": s.sp,
            "status": s.status,
            "opcode": list(s.opcode),
            "len": s.len,
            "mnemonic": s.mnemonic,
        }

    def tick(self) -> None:
        # The bridge's ``step`` advances a single CPU clock cycle.
        self.nes.step()

    def step_instruction(self, limit: int = 4096) -> None:
        """Tick until the next instruction has executed (PC moves)."""
        before = self.nes.snap().pc
        for _ in range(limit):
            self.nes.step()
            if self.nes.snap().pc != before:
                return


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
    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("NES CPU Test")
        self.setMinimumSize(720, 480)
        self.backend = Backend()
        self.running = False

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
        self.cycles_per_tick = QSpinBox()
        self.cycles_per_tick.setRange(1, 1_000_000)
        self.cycles_per_tick.setValue(2000)

        self.load_btn.clicked.connect(self.on_load)
        self.step_btn.clicked.connect(self.on_step)
        self.run_btn.clicked.connect(self.toggle_run)

        controls = QHBoxLayout()
        controls.setSpacing(8)
        controls.addWidget(self.load_btn)
        controls.addWidget(self.step_btn)
        controls.addWidget(self.run_btn)
        controls.addWidget(QLabel("cycles/frame"))
        controls.addWidget(self.cycles_per_tick)
        controls.addStretch(1)
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
        self.disasm = QPlainTextEdit()
        self.disasm.setReadOnly(True)
        self.disasm.setFont(mono)
        self.disasm.setPlaceholderText("last executed instruction")

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
    def on_load(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self, "Open ROM", "", "NES ROM (*.nes)"
        )
        if path:
            self.backend.load(path)
            self.refresh()

    def on_step(self) -> None:
        self.backend.step_instruction()
        self.refresh()

    def toggle_run(self) -> None:
        self.running = not self.running
        self.run_btn.setText("Pause" if self.running else "Run")
        if self.running:
            self.timer.start()
        else:
            self.timer.stop()

    def on_tick(self) -> None:
        for _ in range(self.cycles_per_tick.value()):
            self.backend.tick()
        self.refresh()

    # --- views ---
    def refresh(self) -> None:
        s = self.backend.snap()

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

        self.update_disasm(s)

    def update_disasm(self, s: dict) -> None:
        n = max(int(s["len"]), 1)
        raw = " ".join(f"{b:02X}" for b in s["opcode"][:n])
        self.disasm.setPlainText(f"{raw:<9} {s['mnemonic']}")


def main() -> None:
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    raise SystemExit(app.exec())


if __name__ == "__main__":
    main()
