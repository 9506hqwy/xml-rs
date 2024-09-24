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

### XML Editor Command Line Tool

`xe` is command that edit XML file (UTF-8/no-BOM) using XPATH.

```
xe [--setns xmlns:<prefix>=<uri>]* [<file path>]? --xpath <EXPR> --value <NODE>

    --setns: Specify XML namespace for <EXPR>.
    --xpath: Specify XPATH expression.
    --value: Specify XML node.
    file path: Specify XML file path. (Default: stdin)
```

e.g.

```powershell
PS > write-output "<root><e>text</e></root>" | xe.exe --xpath "root/e" --value 'text2'
<root><e>text2</e></root>
```

```powershell
PS > write-output "<root><e a='b'>text</e></root>" | xe.exe --xpath "root/e/@a" --value 'c'
<root><e a="c">text</e></root>
```

```powershell
PS > write-output "<root><e>a</e><e>b</e><e>c</e></root>" | xe.exe --xpath "root/e" --value 1
<root><e>1</e><e>1</e><e>1</e></root>
```

```powershell
PS > write-output "<root><e>text</e></root>" | xe.exe --xpath "root/e" --value '<ee a="b">text</ee>'
<root><e><ee a="b">text</ee></e></root>
```

## References

* [XML Information Set (Second Edition)](https://www.w3.org/TR/2004/REC-xml-infoset-20040204/)
* [Extensible Markup Language (XML) 1.0 (Fifth Edition)](https://www.w3.org/TR/2008/REC-xml-20081126/)
* [Namespaces in XML 1.0 (Third Edition)](https://www.w3.org/TR/2009/REC-xml-names-20091208/)
* [XML Path Language (XPath) Version 1.0](https://www.w3.org/TR/1999/REC-xpath-19991116/)
* [Document Object Model (DOM) Level 1 Specification](https://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/)
