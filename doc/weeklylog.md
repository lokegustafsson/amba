# Weekly log

## LP3LV1 2023-01-16 to 2023-01-22
# TODO
# DONE
- Tuesday all: Introduction, first meeting.
- Tuesday Loke: Has written some documentation (design.md, technology.md)
- Thursday all: Group meeting. Consultation times booked.
- Thursday Loke: proposes S2E (https://s2e.systems/). Decision to be made next
		monday. Decided on a reading list until monday.

## LP3LV2 2023-01-23 to 2023-01-29
# TODO
# DONE
- Monday all: Meeting Supervisor (Iulia). Decided on going with S2E and not
		build an engine by ourself.
- Monday Enaya: Set up LaTeX template, project plan & report.
- Thursday Samuel: Setting up devenv
- Thursday Loke: Started working on S2E building
- Thursday all: Planning to start with project plan. Assigned
		project plan headings among group members.

## LP3LV3 2023-01-30 to 2023-02-05
# TODO
# DONE
- Monday Loke: More work on S2E building
- Monday Enaya: GANTT chart template
- Monday all: Group meeting. Worked on GANTT chart.
- Thurs all: Group meeting. Discussed report + ethics.
- All: Struggled with writing.

## LP3LV4 2023-02-06 to 2023-02-12
# TODO
# DONE
- Monday all: Writing on project plan
- Monday all: giving feedback to eachother on the writing.
- Thursday all: Written on project plan and taken feeback into account.
- Friday all: Finalized project plan

## LP3LV5 2023-02-13 to 2023-02-19
# TODO
# DONE
- Monday Samuel: Bindings fixed
- Thursday Loke: Libs2e is now compiling, there are still some issues with other 
	build targets.
- Thursday Samuel: Added CI to our git repo and merged pull requests
- Thursday Samuel: Looked around for suitable GUIs and set up relm
- Thursday Enaya: Set up a vm to build s2e-env (tbd if it works)
- Monday Linus: Worked on building s2e with docker (didn't work "out of the
	box")
- Monday Albin: fixed mailing list
- Monday all: added suggestions to our "demo list"

## LP3LV6 2023-02-20 to 2023-02-26
# TODO
* Linus: Namnge nya relevanta rubriker för samtliga avsnitt från
	projektplaneringen och Flytta relevant innehåll till
	"rätt" avsnitt (se förslag på struktur till
	slutrapporten)
* Albin: Splitta upp problem i relevanta delar (klura på ett finurligt
	att göra detta)
* Albin, Clara, Linus: Hitta ett relevant paper vardera och skriv en
	summary (se 2023-02-15.md)
* Se över feedback från projektplaneringen (von hachts feedback) och
	kolla vad vi kan skriva om eller utveckla (ish)
	** Clara: Utveckla syfte (förstå nyttan och det
                  akademiskt relevanta i arbetet)
	** Linus: Beskriv hur avgör vi om syftet är uppnått och
		hur vi går tillväga för att uppnå
		detta (genomförande-del-ish (kopplat
		till vad är ett lyckat demo?))
* Albin: fixa mapphierarki för dokument i git repo
* Clara: Skriva på teoridel, tydliggöra olika begrepp, koncept och
	metoder specifika till vårt projekt
* Albin: Konkretisera krav på prototyp -- hur definierar vi ett lyckat
	demo?  (inte nödvändigtvis rapportspecifikt)
* Linus: Tydliggör vad ett demo egentligen är och ytterligare utveckla
	hur vi använder demo för hur vi uppnår vårt syfte (med
	applikationen)
* Background
	Motivate reverse engineering
	Add a number of real world use cases (the clickbaitier, the better)
	Example that specifically motivates the need for a
	manual automatic hybrid approach
* Cohesive theory section
	** Begrepp
* Split repo in two to move meeting notes out of development?
	Yay rebasing
* Weekly log should contain c/p todo:s from meeting notes.
* Write angry rant email to Wolfgang about Freedom of information and
  fundamental human rights
	Public repo
	Public commit history and/or `weeklylog.md`
* Linus skickar in, till både biblioteket och andra gruppen.
* Läs deras rapport innan mötet på tors.
* Loke: Titta på förbyggde guestimages.
# DONE
* Samuel: Har jobbat med upstream. Fallet jobbas på upstream. Testfallet minimeras, tar tid
	** https://github.com/google/autocxx/issues/1238
* Loke: Har jobbat med att bygga guestimages. Kan bygga kernels och
	starta en ubuntuinstallation i qemu
	** Headless qemu fastnar under installation av ubuntu. Minimal serial output
	** Grafisk qemu klarar installation, men libs2e init misslyckas
* Enaya: Har försökt jobba med s2e-env. Fastnade snabbt
* Albin, Clara, Linus:
	** Skrivit på rapporten
	** Läst relevanta papers (ej summerade än)

## LP3LV7 2023-02-27 to 2023-03-05
# TODO
- Skicka projektrapport till Von-Hacht idag.

- Uppdatera syfte. Etablerat core syfte? - Överträffa existerande
  binäranalysverktyg genom att kombinera olika metoders styrkor.

- Exempel kod i bakgrunden? Där vi förklarar hur olika metoder skulle angripa
  exemplet.

- Jämför i bakgrund exv
	* manuell metod där man kan tydligt se att det är en hashfunktion 
	* automatisk metod (fuzzer)

- The three points we need to adress in Project plan
	* Why analyze binary progams is a thing you may want to do? - to find memory vunrability.
	* What are we building? - Binary analysis tool that combines manual and automatic methods.
	* How evaluate success? - Qualititive comparison on how our vs other tools handle CGC binaries.

- Albin: Mejlar projekt plan till Joachim.
- Idag, om vi inte är överrens kör vi inlämning senare.

- Linus: Exempel kod i bakgrunden, case study bakgrund.
	* What are we building? - Binary analysis tool that combines manual and automatic methods.
- Alla: samläs och samarbeta så att texten blir kompatibel.

- Linus och Albin: Jobba på mid-term presentation. (Förberedelser börjar på Torsdag.)

- Albin: Bakgrund etc.

- Clara: 
	* Why analyze binary progams is a thing you may want to do? - to find memory vunrability.

- Enaya:
	* How evaluate success? - Qualititive comparison on how our vs other tools handle CGC binaries.

- Samuel: Fix caching
- Samuel: Implement demo (identifiera en buffer overrun)
- Loke: Get us closer to running demos.
- Loke: Set up more Clippy rules to catch common Rust mistakes?

# DONE
- Loke: Guest-images i nix-app
- Loke: Rust binär som kör "allt", behöver testas
- Loke: WIP: Guest-images sandboxat inuti nix (borde vara mergebart ikväll).
- Linus: Skrev om bland annat någon mening i syftet och förtydligade
  metod/evaluering av syfte.
- Clara: motivera reverse engineering bättre i bakgrund?
- Clara: Kanske hitta vad metoden (demon osv) ligger under någon typ en
  etablerat metod (Agile manifesto?).
- Albin: Skydda mappar, funkade ej, bara skyddade branches (so you can't push
  code, unless through a pull-request)
- Enaya: Try learning C++ stuff

## LP3LV8 2023-03-06 to 2023-03-12
# TODO
# DONE

Tuesday: Mid-term presentation

## LP3TV 2023-03-13 to 2023-03-19
# TODO
# DONE

## LP4LV1 2023-03-20 to 2023-03-26
# TODO
# DONE

## LP4LV2 2023-03-27 to 2023-04-02
# TODO
# DONE

## LP4 Easter 2023-04-03 to 2023-04-09
# TODO
# DONE

## LP4LV3 2023-04-10 to 2023-04-16
# TODO
# DONE

## LP4LV4 2023-04-17 to 2023-04-23
# TODO
# DONE

## LP4LV5 2023-04-24 to 2023-04-30
# TODO
# DONE

## LP4LV6 2023-05-01 to 2023-05-07
# TODO
# DONE

## LP4LV7 2023-05-08 to 2023-05-14
# TODO
# DONE

## LP4LV8 2023-05-15 to 2023-05-21
# TODO
# DONE

## LP4TV 2023-05-22 to 2023-05-28
# TODO
# DONE
