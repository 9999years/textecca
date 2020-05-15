# textecca

("text rebecca")

I don't like LaTeX. It's decent at what it does, but the system is old, the
developer experience is poor, and it's filled with bad error messages,
counterintuitive behavior, etc.

Textecca is my attempt to fix some of those problems, and explore novel
document organization techniques.

## Output

Textecca will first output HTML/CSS/Javascript(?) documents.

Textecca is web-first -- I view most of my documents on a computer, as do you.
Let's take advantage of the web. Printing is rare, so I shouldn't be staring at
PDFs all day. That said, I want ebook exports to be easy, and I want printing
to be _possible_. But my bandwidth is limited, so it's not a top priority.

I really _like_ the idea of pluggable back-ends (I think [Sphinx] does this?)
but that again introduces a ton of complexity.

## Math

I'm willing to focus textecca on math typesetting pretty heavily, but I don't
want to overly privilige a specific domain within textecca's grammar.

## Symbols

We can (generally) use the following symbols for semantics in a programming
language:

    Text:      !?.,;:'"/-
    Brackets:  {}[]()<>
    Misc:      `~@#$%^&_|\
    Math:      *+=

That's not a ton, especially considering a few of them already have pretty
strong meanings in markup languages/general text. In specific, we probably want
the following symbols to be usable without any escaping in running text:

    !?.,;:/-'"

That doesn't necessarily mean, however, that there can't be _some_ places where
those symbols have other meanings. (For example, I'm fine repurposing single
quotes to render primes in equations.)

Default symbol meanings (tentative):

    {} |
    [] |
    () |
    <> |
    `  | Literal / code
    ~  |
    @  |
    #  | Sectioning
    $  | Math
    %  |
    ^  |
    &  |
    _  |
    |  |
    \  |
    *  | Bold/italics
    +  |
    =  |

## DSLs

I want writing DSLs (domain specific languages -- like TeX's math-mode, or the
hacked together DSLs of [TikZ]) to be easier. In the domain of typesetting, a
DSL ends up being a source-to-source compiler, so I say _easier_ rather than
_easy_.

But I want DSLs to be powerful and useful. I don't entirely know what this will
look like, yet, though.

## Cross-references

I want cross-references to be _much_ easier, and I want a built-in notion of a
"concept" (which can be defined or refered to).

[sphinx]: https://www.sphinx-doc.org/
[tikz]: https://www.ctan.org/pkg/pgf
