.PHONY: all check
all: check group_64_project_report.pdf

check:
	latexindent --local --overwriteIfDifferent --silent -k *.tex */*.tex *.bib
	-rm *.bak0
	-rm */*.bak0
	chktex --nowarn 32 main

group_64_project_report.pdf: \
  amba/main.tex \
  begreppslista.tex \
  cover.tex \
  evaluering/main.tex \
  evaluering/amba-vs-angr.tex
  evaluering/amba-vs-ghidra.tex \
  evaluering/amba-vs-symnav.tex \
  evaluering/arbetsprocess.tex \
  evaluering/vidareutveckling.tex \
  existerande_verktyg/dynamiska_binaranalysramverk.tex \
  existerande_verktyg/main.tex \
  existerande_verktyg/statisk_disassemblering.tex \
  existerande_verktyg/automatiska_fuzzers.tex \
  existerande_verktyg/visualiserad.tex \
  implementation/main.tex \
  inledning/avgransningar.tex \
  inledning/main.tex \
  inledning/syfte.tex \
  inledning/tidigarearbete.tex \
  main.tex \
  slutsats/main.tex \
  teori/arkitektur_for_binaranalysverktyg.tex \
  teori/main.tex \
  teori/symbolisk_exekvering.tex \
  teori/fuzzing.tex \
  titlepage.tex \
  refs.bib
	tectonic main.tex
	mv main.pdf group_64_project_report.pdf
