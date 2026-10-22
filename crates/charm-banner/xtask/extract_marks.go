// Gera crates/charm-banner/src/marks.rs com os intervalos da categoria Unicode
// Mn (marcas diacriticas de combinacao) usados por `normalize`.
//
// A tabela sai da propria biblioteca `unicode` do Go, para que a remocao de
// marcas seja identica a `unicode.Is(unicode.Mn, r)` do banner original.
//
// Uso (a partir da raiz do workspace):
//
//	go run crates/charm-banner/xtask/extract_marks.go crates/charm-banner/src/marks.rs
//
// Este arquivo e um auxiliar de build: nao faz parte da biblioteca publicada.
package main

import (
	"fmt"
	"os"
	"strings"
	"unicode"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "uso: extract_marks <marks.rs>")
		os.Exit(2)
	}

	var b strings.Builder
	b.WriteString("//! Unicode general category `Mn` (non-spacing marks) ranges.\n")
	b.WriteString("//!\n")
	b.WriteString("//! **Generated file - do not edit by hand.**\n")
	b.WriteString("//!\n")
	b.WriteString("//! Extracted from the Go standard library (`unicode.Mn`) by\n")
	b.WriteString("//! `crates/charm-banner/xtask/extract_marks.go`, so the removal of\n")
	b.WriteString("//! combining marks matches the Go original exactly.\n")
	b.WriteString("//!\n")
	b.WriteString("//! Regenerate with:\n")
	b.WriteString("//!\n")
	b.WriteString("//! ```text\n")
	b.WriteString("//! go run crates/charm-banner/xtask/extract_marks.go crates/charm-banner/src/marks.rs\n")
	b.WriteString("//! ```\n\n")
	b.WriteString("/// Sorted, non-overlapping inclusive ranges of the `Mn` category.\n")
	b.WriteString("#[rustfmt::skip]\n")
	b.WriteString("pub(crate) static MN_RANGES: &[(u32, u32)] = &[\n")

	emit := func(lo, hi uint32) {
		fmt.Fprintf(&b, "    (0x%X, 0x%X),\n", lo, hi)
	}
	for _, r := range unicode.Mn.R16 {
		addRange(uint32(r.Lo), uint32(r.Hi), uint32(r.Stride), emit)
	}
	for _, r := range unicode.Mn.R32 {
		addRange(r.Lo, r.Hi, r.Stride, emit)
	}
	b.WriteString("];\n")

	if err := os.WriteFile(os.Args[1], []byte(b.String()), 0o644); err != nil {
		fmt.Fprintln(os.Stderr, "erro:", err)
		os.Exit(1)
	}
	fmt.Printf("`%s` gerado a partir de unicode.Mn\n", os.Args[1])
}

// addRange emite um intervalo contiguo, ou um ponto por codigo quando o passo
// (stride) da tabela e maior que 1.
func addRange(lo, hi, stride uint32, emit func(lo, hi uint32)) {
	if stride <= 1 {
		emit(lo, hi)
		return
	}
	for c := lo; c <= hi; c += stride {
		emit(c, c)
	}
}
