.PHONY: all check
all: check group_64_project_report.pdf

check:
	latexindent --local --overwriteIfDifferent --slient -k *.tex */*.tex *.bib
	-rm *.bak0
	-rm */*.bak0
	chktex --nowarn 32 main

group_64_project_report.pdf: \
  amba/main.tex \
  begreppslista.tex \
  cover.tex \
  abstract.tex \
  sammandrag.text \
  evaluering/main.tex \
  existerande_verktyg/dynamiska_binaranalysramverk.tex \
  existerande_verktyg/main.tex \
  existerande_verktyg/statisk_disassemblering.tex \
  implementation/main.tex \
  inledning/avgransningar.tex \
  inledning/main.tex \
  inledning/syfte.tex \
  inledning/tidigarearbete.tex \
  main.tex \
  slutsats/main.tex \
  teori/arkitektur_for_binaranalysverktyg.tex \
  teori/main.tex \
  teori/statisk_dynamisk_binaranalys.tex \
  teori/symbolisk_exekvering.tex \
  teori/fuzzing.tex \
  titlepage.tex \
  refs.bib
	tectonic main.tex
	mv main.pdf group_64_project_report.pdf
