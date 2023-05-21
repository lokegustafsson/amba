.PHONY: all check
all: group_64_project_presentation.pdf

check:
	latexindent --local --overwriteIfDifferent --silent -k *.tex
	-rm *.bak0
	chktex --nowarn 32 main

group_64_project_presentation.pdf: \
  amba.tex \
  binaryanalysis.tex \
  demo.tex \
  main.tex \
  motivation.tex \
  potential.tex \
  process.tex \
  conclusion.tex \
  symbex.tex
	tectonic main.tex
	mv main.pdf group_64_project_presentation.pdf
