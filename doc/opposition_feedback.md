# Feedback

## 1

Jag vilja påstå att introduktionen är något för teknisk, med tanke på att den bör vara tillgänglig
och förståelig för alla

Det sista stycket i 1.1 syfte känns mer som något som hör hemma i bakgrunden snarare än i syftet.

Jag hade gärna velat se ett avsnitt om de avgränsningar som gruppen satt på projektet och varför ni
behövde göra de avgränsningarna som ni gjort

Implementation och Teori är stundvis svår att följa med, vilket dock är förståeligt projektets
väldigt tekniska natur.

Referenslistan är trasig på slutet, huggs av.

Någon referens är trasig ("??")

Ligger kvar två "TODO" samt en placeholderfigur

I slutsatsen förklaras det att några mål inte har uppnåtts och varför. Detta känns mer rimligt att
ha i evalueringen då några av dessa mål inte hade tagits upp i syftet, vilket slutsatsen ska knyta
an till.

En och ett förväxlas.

Engelska ord med egentlig svensk översättning används, typ "branch", "upstreaming", "fork".

Existerande verktyg har bra (inte för tekniskt) språk.

Vissa extremt långa meningar, tex

> I detta fall är det dessutom orimligt att tillämpa automatiska metoder eftersom dessa, såsom
> konkolisk testning, genererar alltför stora symbo- liska representationer och hade i det ovan
> exemplet krävt att det går att hitta inversen till en given sha256-hash vilket idag är omöjligt och
> leder därmed till att alla stigar i programflödet inte undersöks.

## 2

Kapitel 1.2 om rapportstruktur är meningslöst. Ta bort.

Figurerna och deras första referenser placeras ibland i fel ordning.

Placeholder/TODO. Korrekturläs. **Bristande språk**.

## 3

Placeholder/TODO. Korrekturläs. **Bristande språk**.

Ny sida innan kapitel.

Kapitel 1.2 om rapportstruktur är meningslöst. Ta bort. Innehållsförteckning finns ju.

Avgränsning direkt efter syftet saknas.

Ransomware i inledningen är bra exempel. Men inledning är för teknisk, går inte att förstå för
gemene man.

Vill se kodstycken i implementationen.

Stycket om C++ och autocxx har skum plats. Flytta till nånting annat än evaluering?

Oförklarade ord: "GUI", "musl".

## 4

Inledningen och syftet är bra!

"Det finns ingen metod som kan återskapas"

Nämns inte hur AMBA löser stigexplosionsproblemet

Oklart hur verktyget är tänkt att användas. Ransomware och liknande.

Engelska termer ska föredras över svenska!

Diskussionen och resultatdelen skulle kunna vara mer djupgående.

## 5

Fyra ord i en medning som delas av en helsida med figurer.

Referenser nedåt från introduktionen.

Fuzzing förklaras inte innan syftet.

Diskuterar inte om ni uppnått "förbättra kommunikationen till användaren".

Föredrar svenska översättningar av engelska facktermer.

Ni översätter vissa ord som är tämligen självklara.

Gillar rapportstruktur!

Förkortningen AMBA förklaras inte. Är det förkortning? Förkortningar har endast stor första bokstav.

Evaluering nämner inte att AMBA inte är praktiskt användbart, så skumt att dra den slutsatsen i
evaluering.

GitHubrepot kan refereras till inom referenssystemet.

## 6

Korta ned begreppslistan. 5 sidor är overkill.

Inga avgränsningar direkt efter syftet.

Inledningen förklarar att stigexplosion finns, men inte vad det är. Det kommer först i teorin.

Syftet trycker inte tillräckligt hårt på *interaktiv visualisering*:s-biten.

AMBA kommer för sent, tråkigt att läsa existerande verktyg innan.

AMBA har oproportioneligt många stavfel/grammatiska fel.

Implementationen förklarar på hög nivå, lite för abstrakt. Kan någon nyckelsak förklaras med
kodlisting?

## Kommentarer

Rapportstruktur är väl visst bra!

Kodstycken i implementationen hade varit för detaljnivå, detta är övergripande. Alldeles för mycket
kod för att förklara.

"Det finns ingen metod som kan återskapas" lolwtf

Stigexplosionsproblemet: behöver nämna att vi typ inte angriper det. (Avgränsningar efter syftet?)

Ska man verkligen "referera" till GitHubrepot?

Man behöver känna till existerande verktyg innan AMBA känns motiverat. Liksom man kan skippa till
AMBA om man är intresserad av det konkreta.

# TODOs

## Uppenbara TODOs

- Behöver projektavgränsningar mellan syfte och rapportstruktur. Nämna arkitekturer, vi använder
	färdig motor, stigexplosionslösning inte huvudfokus.
- Avhuggen referenslista
- "??"-referenser
- En och ett förväxlas
- Vissa extremt långa meningar
- Figurerna och deras första referenser placeras ibland i fel ordning (kanske 2.4 schematisk bild?,
	eller framförallt 4.1-3 AMBA?)
- Ny sida innan kapitel.
- Oförklarade ord: "GUI", "musl".
- Ligger kvar två "TODO" samt en placeholderfigur
- Fyra ord i en medning som delas av en helsida med figurer.
- Fuzzing förklaras inte innan syftet.

## Frivilliga enkla TODOss

- IEEE-referera till GitHub
- Människa+Dator=Bra ska inte skrivas i syftet (ens upprepas?)
- Svengelska "branch", "upstreaming", "fork"
- Rapportstruktur meningslös?
- Referenser nedåt från introduktionen och annat.
- Förklara "förkortning" AMBA. Förkortning stavas Amba.
- Korta ned begreppslistan
- Inledningen förklarar inte vad stigexplosion är
- Syftet trycker inte tillräckligt hårt på *interaktiv visualisering*:s-biten.
- AMBA kommer för sent, tråkigt att läsa existerande verktyg innan.

## Övriga TODOs

- För teknisk inledning, ska vara förståelig för "alla"
- För teknisk teori
- Flytta vissa saker mellan evaluering/slutsats. Slutsats ska inte introducera nytt utan anknyta
	till evalueringen.
- Implementationen förklarar på hög nivå, lite för abstrakt. Kan någon nyckelsak förklaras med
	kodlisting?
- Stycket om C++ och autocxx har skum plats. Flytta till nånting annat än evaluering?
- "Det finns ingen metod som kan återskapas"
- "Hur är verktyget tänkt att användas?"
- Diskuterar inte om ni uppnått "förbättra kommunikationen till användaren".
- Evaluering nämner inte att AMBA inte är praktiskt användbart, så skumt att dra den slutsatsen i
	evaluering.
