# Textecca

TeX is bad (or, at the very least, old). What do I mean by "bad"? And what's a
better system look like? Here, we'll explore the design space of typesetting
systems.

It's difficult to talk about TeX's flaws without talking about its age.
Computers, software, software development, and user interfaces have changed
enormously in the 42 years since TeX's first release in 1978. [Your computer is
not a fast PDP-11.][c-low-level] Computers have many conveniences that the
first users of TeX made do without, such as "screens".

TeX's design was heavily influenced by the constraints of computers in the
1970s, most of which have largely vanished in a world where most computers have
several cores, GHz clock speeds, gigabytes of RAM, terabytes of persistent
storage, and high-resolution displays. We can now make different time-memory
tradeoffs, leverage concurrency, and more to create a fluid and enjoyable user
experience.

TeX, unfortunately, does not perform a single function. TeX contains, wrapped
into one tangled glob, a register-like virtual machine, a macro-based
programming language and combined parser/evaluator, a layout and justification
engine, and renderer.

The box-and-glue model used by the layout engine and its accompanying
justification algorithm remain best-in-class.

One of the things I want to talk about here is "why is this such a hard
problem?"


## Boxes and Glue

The most durable part of TeX is the venerable box-and-glue model used for
describing layouts in terms of constraints. However, page layout is one of the
least interesting aspects of TeX and document design in general to me --- and,
more precisely, the restrictions of paper pages and designs (non-interactive,
static, un-zoomable) are exactly what I want future systems to go beyond.

## Charm and Humanism

There is something peculiar about Donald Knuth as a person. I think his little
witticisms are very annoying, and I exceptionally despise that the TeX users
group has titled him "Grand Wizard", but there's a certain humanism (intimacy?)
in TeX.

It feels unfinished in a deliberate way. TeX has "whatsit"s and more, little
things that seem to... just not be named.

## Beyond plain text

Using plain text for source code has certain limitations, so we create IDEs or
language servers to add functionality to our text editors. These are tools
which add additional context and parse meaningful information out of source
code; a language server can answer questions like "where is the command under
my cursor defined?" or "where's the documentation for this function?"

These tools become more important when working on larger projects, because they
let users offload [working memory][working-memory] to an automated system — you
don't need to learn all the built-in commands if you can use autocompletion to
see exactly which ones are in scope at any point or hover over them to see
their documentation, along with argument names and counts.

But we can take this concept even further. Language servers paper over plain
text source code by adding a layer of extra contextual information, but what
advantages does plain text bring? Though "integration with preexisting tools"
is a common refrain, downstream consequences of generalized tools are
commonplace: line-based diffs can get noisy, code searchability begins to
depend on formatting choices, and more.

Syntax errors are a common stumbling block for beginners, and they're a result
of plain text source code as well. In plain text, any sequence of characters is
valid, but parsers and compilers only accept a tiny subset of possible inputs.
We can avoid this by allowing our editor to only represent valid programs.
Scratch does this, for instance, by associating each type in the parse tree a
shape: expressions, variables, boolean conditions, and more all have specific
shapes that make it clear how different types of "block" fit together into a
bigger program.

We can leverage this concept to avoid several of TeX's most confusing behaviors
around parsing *while* avoiding the author-side overhead of a more verbose
language.

## TeX's flexibility is a double-edged sword

- TeX is an *incredibly* dynamic language.
- Macro expansion is astonishingly flexible — and makes error messages *really
  bad,* and static analysis largely impossible.
- The design constraints of TeX --- in particular, the need for brevity ---
  makes TeX source largely unreadable.

## DSLs

I believe TeX's support for [DSLs][dsl] is one of its greatest strengths. The
same flexibility that makes the macro system so intimidating, confusing, and
complicated allows package authors to embed nearly any language in TeX, limited
only by the package author's willingness to write code in TeX.

What would DSLs look like in non plain text? Would there be style instructions?
Does defining a DSL turn from a compiler problem to a GUI problem?

If we remove the design constraint that plain-text code must be concise, that
is, by separating the input keystrokes from what the parser and compiler see,
we can create an interface that's less hellish for users, viewers, and
compilers.

I spent so long thinking about syntax, and how syntax can uplift and create
DSLs, but now I don't think plain-text syntax is even the answer, which
transforms the questions of DSLs *dramatically.*

## Concept-based authoring

Documents, and human language in general, are filled with references to other
pieces of information; sometimes in documents, sometimes not. Different sorts
of documents make these references and links more or less explicit ---
Wikipedia is filled with links to other pages and citations, but even the most
thoroughly-researched historical fiction will go to great lengths to hide that
fact.

My working memory and ability to memorize definitions are extremely limited, so
explicit references can be extremely useful to me --- documents that can remind
me of context or relevant definitions are much more accessible.

Traditional paper documents have handled links through various numbering
schemes; textbooks often have numbered chapters, sections, figures, theorems,
examples, and problems, which are all referred to by number throughout the
text. When designing TeX, Knuth created tools for incrementing and displaying
various counters to automate numbering of sections and figures, but didn't take
the concept any further. Later, pdfTeX would enable these numbered references
to be clickable links to other pages, but PDF viewers are bad at preserving
context --- viewing history within a document is opaque and difficult to
navigate, if navigable at all.

Can we take this further? Browser extensions like [WikiWand][wikiwand] show
previews of linked articles without navigating to a separate page, just like
IDEs can often show a function's documentation or implementation in a small
window without losing context --- we should be able to access linked context in
the same way within written documents.

How else can we take advantage of browsers? One area that physical paper excels
at is note-taking --- paper offers virtually no constraints on formatting, so
organization is free-form and up to the individual note-taker. PDF readers and
browsers rarely offer good built-in options for note-taking, as well. Though
some office software suites offer decent commenting and reviewing facilities,
they're often strained by long-form conversations, difficult to browse, share,
and merge, and are often limited in expressiveness and formatting (it's rare to
be able to annotate in bold or italics, much less add drawings or images, even
though physical paper makes this effortless).

[working-memory]: https://en.wikipedia.org/wiki/Working_memory
[dsl]: https://en.wikipedia.org/wiki/Domain-specific_language
[wikiwand]: https://chrome.google.com/webstore/detail/wikiwand-wikipedia-modern/emffkefkbkpkgpdeeooapgaicgmcbolj
[c-low-level]: https://queue.acm.org/detail.cfm?id=3212479
