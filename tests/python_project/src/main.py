#!/usr/bin/env python3
"""🔥 FORGE — Proyecto Python de Ejemplo"""

import sys
import platform


def main():
    print("========================================")
    print("  🔥 FORGE — Proyecto Python de Ejemplo")
    print("========================================")
    print()
    print("  ✅ ¡La ejecución con FORGE funciona!")
    print("  🐍 Ejecutado via entorno virtual FORGE")
    print("  🦀 Build system escrito en Rust")
    print()
    print(f"  Python Version: {sys.version.split()[0]}")
    print(f"  OS: {platform.system()} {platform.release()}")
    print()
    print("========================================")


if __name__ == "__main__":
    main()
