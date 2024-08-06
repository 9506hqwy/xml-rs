pub mod error;
pub mod eval;
pub mod expr;

pub fn query<'a>(
    dom: xml_dom::XmlDocument,
    expr: &'a str,
    context: &mut eval::model::Context,
) -> error::Result<'a, eval::model::Value> {
    let (rest, q) = expr::parse(expr).map_err(|v| error::Error::ExprSyntax(v.to_string()))?;
    if !rest.is_empty() {
        return Err(error::Error::ExprRemain(rest));
    }

    let v = eval::document(&q, dom, context)?;

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eg_location_path_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<para />").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "child::para", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<a />").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "child::*", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<a></a>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_text() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::text()",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("text1text2", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_node() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::node()",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("text1<para></para>text2", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_attr_name() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root name='a'></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root[attribute::name]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<root name=\"a\"></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_attr_ns() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root name='a'></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root[attribute::*]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<root name=\"a\"></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_descendant_para() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "descendant::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ancestor_div() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "//ancestor::div", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<div><para></para></div>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ancestor_self_div() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "//ancestor-or-self::div",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<div><para></para></div>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_descendant_self_para() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "descendant-or-self::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_self_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para/self::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_chapter_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter><section><para /></section></chapter></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::chapter/descendant::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns_para() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><chapter><para /></chapter></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::*/child::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "/", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_descendant_para() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><chapter><para /></chapter></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "/descendant::para",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_olist_item() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><olist><item /></olist></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "/descendant::olist/child::item",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<item></item>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_1() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[position()=1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_last() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[position()=last()]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_prelast() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[position()=last()-1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_gt_1() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[position()>1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_following_chapter_1() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><para /><chapter>1</chapter><chapter>2</chapter></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para/following-sibling::chapter[position()=1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter>1</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_preceding_chapter_1() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter>1</chapter><chapter>2</chapter><para /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para/preceding-sibling::chapter[position()=1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter>2</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_preceding_chapter_1_rev() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter>1</chapter><chapter>2</chapter><para /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "(root/para/preceding-sibling::chapter)[position()=1]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter>1</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_figure_42() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><figure>1</figure><figure>2</figure></root>")
                .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "/descendant::figure[position()=42]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_chapter_5_section_2() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<doc><chapter>1</chapter><chapter><section>1</section><section>2</section></chapter></doc>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "/child::doc/child::chapter[position()=2]/child::section[position()=2]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<section>2</section>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_warning() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><para type='error' /><para type='warning' /><para type='normal' /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[attribute::type=\"warning\"]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\"></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_warning_5() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[attribute::type='warning'][position()=5]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\">5</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_5_warning() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::para[position()=5][attribute::type=\"warning\"]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\">3</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_chapter_intro() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><chapter><title>Introduction</title></chapter><chapter><title>Second</title></chapter></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::chapter[child::title='Introduction']",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!(
            "<chapter><title>Introduction</title></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_location_path_chapter_title() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter></chapter><chapter><title /></chapter></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::chapter[child::title]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter><title></title></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter /><para><chapter /></para><appendix /><chapter /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::*[self::chapter or self::appendix]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!(
            "<chapter></chapter><appendix></appendix><chapter></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix_last() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter /><para><chapter /></para><appendix /><chapter /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/child::*[self::chapter or self::appendix][position()=last()]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<para></para>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "para", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_ns() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<para></para>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "*", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_text() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root>a</root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/text()", &mut eval::model::Context::default()).unwrap();
        assert_eq!("a", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_attr_name() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root name='a'></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/@name", &mut eval::model::Context::default()).unwrap();
        assert_eq!("name=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_attr_ns() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root name='a'></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/@*", &mut eval::model::Context::default()).unwrap();
        assert_eq!("name=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_1() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>2</para><para>1</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/para[1]", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_last() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root><para>2</para><para>1</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para[last()]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_root_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<para></para>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "/para", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_root_chapter_5_section_2() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<doc><chapter/><chapter/><chapter/><chapter/><chapter><section/><section>section</section><section/></chapter></doc>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "/doc/chapter[5]/section[2]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<section>section</section>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_chapter_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(doc, "chapter//para", &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_root_descendant_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(doc, "//para", &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_root_olist_item() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><para><item>1</item><olist><item>2</item></olist></para><item>3</item></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(doc, "//olist/item", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<item>2</item>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_current() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, ".", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_current_descendant_para() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(doc, ".//para", &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_parent() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/para/..", &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root><para></para></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_parent_attr_lang() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root lang='a'><para /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "//para/../@lang", &mut eval::model::Context::default()).unwrap();
        assert_eq!("lang=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_warning() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><para type='error' /><para type='warning' /><para type='normal' /></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para[@type=\"warning\"]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\"></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_warning_5() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para[@type=\"warning\"][5]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\">5</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_5_warning() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/para[5][@type=\"warning\"]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<para type=\"warning\">3</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_chapter_intro() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><chapter><title>Introduction</title></chapter><chapter><title>Second</title></chapter></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/chapter[title=\"Introduction\"]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!(
            "<chapter><title>Introduction</title></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_chapter_title() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw(
            "<root><chapter></chapter><chapter><title /></chapter></root>",
        )
        .unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/chapter[title]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<chapter><title></title></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_employee() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root><employee secretary='a'/><employee secretary='a' assistant='b' /><employee a='b'/><employee assistant='b'/></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/employee[@secretary and @assistant]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!(
            "<employee secretary=\"a\" assistant=\"b\"></employee>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_namespace_predicate() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/e2[namespace::a]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<e2 xmlns:a=\"http://test/a\"></e2>", format!("{}", r));
    }

    #[test]
    fn test_eg_namespace_value() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/e2/namespace::a",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("xmlns:a=\"http://test/a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_namespace_default() {
        let (rest, doc) = xml_dom::XmlDocument::from_raw("<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/e2[namespace::xml]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!(
            "<e2 xmlns:a=\"http://test/a\"></e2><e2></e2>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_text_reference() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>a&amp;b<e1/><![CDATA[c]]></root>").unwrap();
        assert_eq!("", rest);

        let r = query(doc, "root/text()", &mut eval::model::Context::default()).unwrap();
        assert_eq!("a&amp;b<![CDATA[c]]>", format!("{}", r));
    }

    #[test]
    fn test_eg_text_reference_eq() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>a&lt;b<e1/><![CDATA[c]]></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/text()[. = 'c']",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("<![CDATA[c]]>", format!("{}", r));
    }

    #[test]
    fn test_eg_text_reference_contains() {
        let (rest, doc) =
            xml_dom::XmlDocument::from_raw("<root>a&lt;b<e1/><![CDATA[c]]></root>").unwrap();
        assert_eq!("", rest);

        let r = query(
            doc,
            "root/text()[contains(., '<')]",
            &mut eval::model::Context::default(),
        )
        .unwrap();
        assert_eq!("a&lt;b", format!("{}", r));
    }

    #[test]
    fn test_eg_subtraction() {
        let (rest, _) = expr::parse("foo-bar").unwrap();
        assert_eq!("", rest);

        let (rest, _) = expr::parse("foo - bar").unwrap();
        assert_eq!("", rest);

        let (rest, _) = expr::parse("foo- bar").unwrap();
        assert_eq!(" bar", rest);

        let (rest, _) = expr::parse("foo -bar").unwrap();
        assert_eq!("", rest);
    }
}
