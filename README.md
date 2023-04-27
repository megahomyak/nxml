# NXML

NXML is a markup language that is **not XML**. It was based on XML, but went in a very different direction and now does not resemble XML at all.

## Why is this a decent language

* Simpler than JSON
* Can be losslessly decoded and encoded using this library
* Very flexible and concise at the same time
* Escape sequences are concise (just `\` and your character), not some incomprehensible bullshit like those in XML

## Why there are no built-in types (apart from sequences and strings)

When was the last time you've been working with structures that required you to interpret them differently based on types of some of their fields? Never? Well, me too, and that's the reason.

## Syntax explanation

* A node is either a text (`hello`) or a sequence (`[I|am|a|sequence]`)
* Brackets (`[` and `]`) surround the sequence
* Nodes can go one after another without any explicit delimiters (`text[sequence][sequence]text`)
* A vertical bar (`|`) denotes the end of a text node (otherwise text nodes end at the end of input or at a sequence boundary (a bracket)). This can be used to place text nodes one after another or to create an empty text node
* Every special character (`|`, `[`, `]` or `\`) can be escaped with a backslash (`\`)

## Syntax grammar (in pseudo-Backus-Naur Form; may be invalid)

    special_character ::= "[" | "]" | "|" | "\\"
    text_character ::= !special_character | "\\\\" | "\\[" | "\\]" | "\\|"
    text ::= text text_character | text_character | text "|" | "|"
    sequence_of_nodes ::= sequence_of_nodes node | node
    node ::= text | "[" + sequence_of_nodes + "]" | "[]"

## Syntax example

    [to-do list|
        [buy some groceries|[
            lettuce |
            cucumber |
            ketchup
        ]]
        [finish NXML|[
            add syntax explanation to the README |
            update syntax examples |
            rewrite the parser
        ]]
        [do the homework]
    ]


    [user|
        [id|123]
        [name|Paul]
        [surname|Brown]
        [profession|Architect]
        [friend ids|[234|345|456]]
    ]


    [
        [message|
            [from|Alice]
            [to|Bob]
            [contents|Hello!]
        ]
        [message|
            [from|Bob]
            [to|Alice]
            [contents|Hi!]
        ]
    ]


    [ARTICLE|
        [TITLE|Cookies are good]
        [CONTENTS|
            [PARAGRAPH|
                [OUTER REFERENCE|[URL|https://en.wikipedia.org/wiki/Cookie][TEXT|Cookies]] are good!
            ]
        ]
    ]


    you can use the sequential parser to parse bare sequential nodes |
    something |
    something


    an empty text node! -> || <- here it is!
    another one, this time inside a sequence: [|] (it is between the opening bracket and the vertical bar)

## Things to consider when using this library

* This library **does not** trim the whitespaces at the end or the beginning of text nodes for better flexibility. It is **the user's responsibility** to trim the excessive whitespaces.
