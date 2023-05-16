.PHONY: all check
all: check group_64_bidragsrapport.pdf

check:
	latexindent --local --overwriteIfDifferent --silent -k *.tex
	-rm *.bak0
	-rm */*.bak0
	chktex --nowarn 32 main

group_64_bidragsrapport.pdf: *.tex
	tectonic main.tex
	mv main.pdf $@
