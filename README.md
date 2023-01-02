# NXML

NXML is a markup language that is **not XML**. It is based on XML, but it is not compatible with XML (and vice versa).

## Differences from XML

* Tag names can contain any characters
* Tags cannot contain attributes
* Special characters are escaped with `\` instead of cryptic sequences

## Syntax example

    [tag:text[another tag][yet another tag:]]

Same in XML (with whitespaces replaced by underscores):

    <tag>text<another_tag /><yet_another_tag></yet_another_tag></tag>
