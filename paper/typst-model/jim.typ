// Template for JIM
//     jim.typ -> Typst template
// Original LaTeX template by Eloi Batlle, Bram de Jong
//     changes for JIM 2007 by Dominique Fober
//     changes for JIM 2009 by Olivier Tache
//     changes for JIM 2021 by Yann Orlarey
//     changes for JIM 2023 by Paul Goutmann
//     changes for JIM 2026 by Mathieu Giraud
// Typst template by Raphaël Maurice Forment, 2026 (Typst 0.14)

#let jim(
  title: "",
  authors: (),
  abstract: [],
  doc,
) = {
  // Page
  set page(
    "a4",
    margin: (top: 20mm, bottom: 30mm, left: 20mm, right: 20mm),
    columns: 2,
    numbering: none,
    header: none,
    footer: none,
  )
  set columns(gutter: 8mm)

  // Fonts
  set text(
    font: "Times New Roman",
    size: 10pt,
    lang: "fr",
  )
  set par(justify: true, first-line-indent: (amount: 12pt, all: true), leading: 0.55em)

  // Disable smart quotes (use straight quotes, not French guillemets)
  set smartquote(enabled: false)

  // Headings
  set heading(numbering: "1.")

  // Level 1: bold, centered, UPPERCASE
  show heading.where(level: 1): it => {
    set text(size: 10pt, weight: "bold")
    set par(first-line-indent: 0pt)
    v(15pt)
    block(width: 100%, above: 0pt, below: 10pt, {
      align(center, {
        counter(heading).display("1.")
        h(0.6em)
        upper(it.body)
      })
    })
  }

  // Level 2: bold, left-aligned
  show heading.where(level: 2): it => {
    set text(size: 10pt, weight: "bold")
    set par(first-line-indent: 0pt)
    v(14pt)
    block(above: 0pt, below: 6.5pt, {
      counter(heading).display("1.1.")
      h(0.6em)
      it.body
    })
  }

  // Level 3: italic, left-aligned
  show heading.where(level: 3): it => {
    set text(size: 10pt, weight: "regular", style: "italic")
    set par(first-line-indent: 0pt)
    v(14pt)
    block(above: 0pt, below: 6.5pt, {
      counter(heading).display("1.1.1.")
      h(0.6em)
      it.body
    })
  }

  // Footnotes
  set footnote.entry(
    separator: line(length: 30%, stroke: 0.5pt),
  )
  show footnote.entry: set text(size: 8pt)

  // Force English figure/table supplements
  show figure.where(kind: image): set figure(supplement: "Figure")
  show figure.where(kind: table): set figure(supplement: "Table")

  // Figures and tables: bold supplement + ". " + body
  show figure.caption: it => {
    v(10pt)
    set par(first-line-indent: 0pt)
    align(left, {
      text(weight: "bold", it.supplement + [ ] + it.counter.display())
      [. ]
      it.body
    })
  }

  // Equations
  set math.equation(numbering: "(1)")

  // Title block spanning both columns
  place(
    top + center,
    float: true,
    scope: "parent",
    clearance: 1.5em,
    {
      set par(first-line-indent: 0pt)
      v(2em)
      // Title
      align(center, text(size: 14pt, weight: "bold", upper(title)))
      v(1.5em)
      // Authors
      if authors.len() > 0 {
        align(center, {
          for (i, author) in authors.enumerate() {
            if i > 0 { h(1in) }
            box({
              align(center, {
                text(style: "italic", author.name)
                linebreak()
                author.affiliation
              })
            })
          }
        })
      }
    },
  )

  // Abstract in left column only
  {
    set par(first-line-indent: 0pt)
    align(center, text(weight: "bold", smallcaps[Résumé]))
    abstract
  }

  doc
}
