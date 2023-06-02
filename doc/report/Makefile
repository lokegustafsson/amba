.PHONY: all check
all: check group_64_project_report.pdf

check:
	latexindent --local --overwriteIfDifferent --silent -k *.tex */*.tex *.bib
	-rm *.bak0
	-rm */*.bak0
	chktex --nowarn 32 main

group_64_project_report.pdf: \
  amba/main.tex \
  amba/varfor_s2e.tex \
  befintliga_verktyg/dynamiska_binaranalysramverk.tex \
  befintliga_verktyg/main.tex \
  befintliga_verktyg/statisk_disassemblering.tex \
  befintliga_verktyg/automatiska_fuzzers.tex \
  befintliga_verktyg/visualiserad.tex \
  begreppslista.tex \
  cover.tex \
  abstract.tex \
  evaluering/main.tex \
  evaluering/amba-vs-angr.tex \
  evaluering/amba-vs-ghidra.tex \
  evaluering/amba-vs-symnav.tex \
  evaluering/arbetsprocess.tex \
  evaluering/vidareutveckling.tex \
  implementation/main.tex \
  inledning/main.tex \
  main.tex \
  slutsats/main.tex \
  tack.tex \
  teori/arkitektur_for_binaranalysverktyg.tex \
  teori/main.tex \
  teori/symbolisk_exekvering.tex \
  teori/fuzzing.tex \
  titlepage.tex \
  refs.bib
	tectonic main.tex
	mv main.pdf group_64_project_report.pdf
