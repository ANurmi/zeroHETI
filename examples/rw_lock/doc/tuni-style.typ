// Sources:
//
// * https://markkinointipankki.tuni.fi/ohjeistukset/graafinen-ohjeistus/
// * https://markkinointipankki.tuni.fi/wp-content/uploads/2024/05/TUNI_Graafinenohjeistus_052024_2.pdf

/*
# Värit

<https://markkinointipankki.tuni.fi/ohjeistukset/varit/>

Digitaalisissa käyttöympäristöissä käytetään aina RGB-määrityksiä. Painotuotteissa pyritään käyttämään Pantone-määrityksiä aina kun se on mahdollista.

Violetti on hallitseva väri, jota kevennetään valkoisella. Violetista voidaan
käyttää myös eri vaaleusasteita. Toissijaista väripalettia käytetään niukasti.

Digitaalisissa käyttöympäristöissä käytetään aina RGB-määrityksiä. Painetussa
mediassa käytetään CMYK tai Pantone -värimäärityksiä.
*/

/* Ensisijaiset värit */
#let tuni-purple = oklch(35.54%, 0.19, 299.3deg)
#let tuni-white = oklch(100%, 0, 90deg)
#let tuni-black = oklch(0%, 0, 0deg)

/* Toissijaiset värit */
#let tuni-blue = oklch(80.11%, 0.09, 234.58deg)
#let tuni-pink = oklch(80.97%, 0.104, 351.81deg)
#let tuni-yellow = oklch(91.1%, 0.08, 78.72deg)
#let tuni-lpurple = oklch(80.35%, 0.043, 300.63deg)
#let tuni-fuchsia = oklch(70.72%, 0.154, 11.89deg)
#let tuni-green = oklch(79.25%, 0.082, 180.8deg)
#let tuni-grey = oklch(83.28%, 0, 180deg)

/*
# Saavutettavat värit

- Valkoinen tausta: violetti, musta ja fuchsia
- Violetti tausta: tuniBlue, tuniPink, tuniYellow, tuniGreen, tuniGrey, tuniWhite
- Toissijainen väripaletti taustana: violetti ja musta
- Valkoinen teksti on saavutettava ainoastaan tuniFuchsian päällä
*/

/*
# Typografia

## Digitaaliset ympäristöt

Open Sans -fonttia käytetään kaikissa digitaalisissa järjestelmissä silloin kun
se on mahdollista. Open Sans on Googlen ilmainen kirjaisinperhe ja sen voi
ladata osoitteesta: fonts.google.com.

Kirjaisinperheen kaikki leikkaukset ovat käytettävissä.

Office-ympäristö

Office-ohjelmissa (Word, PowerPoint, Excell) Neue Haas Unica -fontin tilalla on
Arial ja sen kaikki leikkaukset. Arial voi soveltua myös digitaalisten
järjestelmien käyttöön.
*/

#let tuni-font = "Open Sans"
#let tuni-font-ms = "Arial"

/*
Kirjasinkoko

- Minimi 11 on saavutettava.
  - Poikkeus: kaavioiden arvopisteiden otsikot voivat olla 9.
*/

#let tuni-font-size = 12pt // 16 px
#let tuni-font-size-graph-min = 9pt
#let tuni-font-size-code = 10pt // 13px
#let tuni-codeblock-inset = 0.5em
