# XML for Rust

## Sample Application

### XML Query Command Line Tool

`xq` is command that query XML file (UTF-8/no-BOM) using XPATH.

```
xq [--setns xmlns:<prefix>=<uri>]* [<file path>]? --xpath <EXPR>

    --setns: Specify XML namespace for <EXPR>.
    --xpath: Specify XPATH expression.
    file path: Specify XML file path. (Default: stdin)
```

e.g.

```powershell
PS > write-output "<root><e>text</e></root>" | xq.exe --xpath "root/e/text()"
text
```

```powershell
PS > write-output "<root xmlns:abc='http://abc'><abc:e>text</abc:e></root>" | xq.exe --setns "xmlns:i=http://abc"  --xpath "root/i:e"
<abc:e>text</abc:e>
```

## References

* [XML Information Set (Second Edition)](https://www.w3.org/TR/2004/REC-xml-infoset-20040204/)
* [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/2008/REC-xml-20081126/)
* [Namespaces in XML 1.0 (Third Edition)](https://www.w3.org/TR/2009/REC-xml-names-20091208/)
* [XML Path Language (XPath) Version 1.0](https://www.w3.org/TR/1999/REC-xpath-19991116/)
* [Document Object Model (DOM) Level 1 Specification](https://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/)
