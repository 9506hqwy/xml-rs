pub mod eval;
pub mod expr;

// FIXME: Entity Reference to test node.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eg_location_path_para() {
        let (rest, tree) = xml_parser::document("<para />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("child::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns() {
        let (rest, tree) = xml_parser::document("<a />").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("child::*").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<a></a>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_text() {
        let (rest, tree) = xml_parser::document("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::text()").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("text1text2", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_node() {
        let (rest, tree) = xml_parser::document("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::node()").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("text1<para></para>text2", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_attr_name() {
        let (rest, tree) = xml_parser::document("<root name='a'></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root[attribute::name]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root name=\"a\"></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_attr_ns() {
        let (rest, tree) = xml_parser::document("<root name='a'></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root[attribute::*]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root name=\"a\"></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_descendant_para() {
        let (rest, tree) = xml_parser::document("<root>text1<para />text2</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("descendant::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ancestor_div() {
        let (rest, tree) = xml_parser::document("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("//ancestor::div").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<div><para></para></div>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ancestor_self_div() {
        let (rest, tree) = xml_parser::document("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("//ancestor-or-self::div").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<div><para></para></div>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_descendant_self_para() {
        let (rest, tree) = xml_parser::document("<root><div><para /></div></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("descendant-or-self::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_self_para() {
        let (rest, tree) = xml_parser::document("<root><para /></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para/self::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_chapter_para() {
        let (rest, tree) =
            xml_parser::document("<root><chapter><section><para /></section></chapter></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::chapter/descendant::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns_para() {
        let (rest, tree) =
            xml_parser::document("<root><chapter><para /></chapter></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::*/child::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root() {
        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_descendant_para() {
        let (rest, tree) =
            xml_parser::document("<root><chapter><para /></chapter></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/descendant::para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_olist_item() {
        let (rest, tree) = xml_parser::document("<root><olist><item /></olist></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/descendant::olist/child::item").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<item></item>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_1() {
        let (rest, tree) =
            xml_parser::document("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::para[position()=1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_last() {
        let (rest, tree) =
            xml_parser::document("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::para[position()=last()]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_prelast() {
        let (rest, tree) =
            xml_parser::document("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::para[position()=last()-1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_gt_1() {
        let (rest, tree) =
            xml_parser::document("<root><para>1</para><para>2</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::para[position()>1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_following_chapter_1() {
        let (rest, tree) =
            xml_parser::document("<root><para /><chapter>1</chapter><chapter>2</chapter></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("root/para/following-sibling::chapter[position()=1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter>1</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_preceding_chapter_1() {
        let (rest, tree) =
            xml_parser::document("<root><chapter>1</chapter><chapter>2</chapter><para /></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("root/para/preceding-sibling::chapter[position()=1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter>2</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_preceding_chapter_1_rev() {
        let (rest, tree) =
            xml_parser::document("<root><chapter>1</chapter><chapter>2</chapter><para /></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("(root/para/preceding-sibling::chapter)[position()=1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter>1</chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_figure_42() {
        let (rest, tree) =
            xml_parser::document("<root><figure>1</figure><figure>2</figure></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/descendant::figure[position()=42]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_root_chapter_5_section_2() {
        let (rest, tree) =
            xml_parser::document("<doc><chapter>1</chapter><chapter><section>1</section><section>2</section></chapter></doc>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("/child::doc/child::chapter[position()=2]/child::section[position()=2]")
                .unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<section>2</section>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_warning() {
        let (rest, tree) = xml_parser::document(
            "<root><para type='error' /><para type='warning' /><para type='normal' /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::para[attribute::type=\"warning\"]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\"></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_warning_5() {
        let (rest, tree) =
            xml_parser::document("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("root/child::para[attribute::type='warning'][position()=5]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\">5</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_para_5_warning() {
        let (rest, tree) =
            xml_parser::document("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("root/child::para[position()=5][attribute::type=\"warning\"]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\">3</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_chapter_intro() {
        let (rest, tree) =
            xml_parser::document("<root><chapter><title>Introduction</title></chapter><chapter><title>Second</title></chapter></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::chapter[child::title='Introduction']").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<chapter><title>Introduction</title></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_location_path_chapter_title() {
        let (rest, tree) =
            xml_parser::document("<root><chapter></chapter><chapter><title /></chapter></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::chapter[child::title]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter><title></title></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix() {
        let (rest, tree) = xml_parser::document(
            "<root><chapter /><para><chapter /></para><appendix /><chapter /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/child::*[self::chapter or self::appendix]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<chapter></chapter><appendix></appendix><chapter></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix_last() {
        let (rest, tree) = xml_parser::document(
            "<root><chapter /><para><chapter /></para><appendix /><chapter /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) =
            expr::parse("root/child::*[self::chapter or self::appendix][position()=last()]")
                .unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para() {
        let (rest, tree) = xml_parser::document("<para></para>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_ns() {
        let (rest, tree) = xml_parser::document("<para></para>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("*").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_text() {
        let (rest, tree) = xml_parser::document("<root>a</root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/text()").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("a", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_attr_name() {
        let (rest, tree) = xml_parser::document("<root name='a'></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/@name").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("name=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_attr_ns() {
        let (rest, tree) = xml_parser::document("<root name='a'></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/@*").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("name=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_1() {
        let (rest, tree) =
            xml_parser::document("<root><para>2</para><para>1</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para[1]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>2</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_last() {
        let (rest, tree) =
            xml_parser::document("<root><para>2</para><para>1</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para[last()]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para>1</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_root_para() {
        let (rest, tree) = xml_parser::document("<para></para>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_root_chapter_5_section_2() {
        let (rest, tree) = xml_parser::document("<doc><chapter/><chapter/><chapter/><chapter/><chapter><section/><section>section</section><section/></chapter></doc>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("/doc/chapter[5]/section[2]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<section>section</section>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_chapter_para() {
        let (rest, tree) = xml_parser::document(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("chapter//para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_root_descendant_para() {
        let (rest, tree) = xml_parser::document(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("//para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_root_olist_item() {
        let (rest, tree) = xml_parser::document(
            "<root><para><item>1</item><olist><item>2</item></olist></para><item>3</item></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("//olist/item").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<item>2</item>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_current() {
        let (rest, tree) = xml_parser::document("<root></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse(".").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_current_descendant_para() {
        let (rest, tree) = xml_parser::document(
            "<chapter><para>1</para><a><para>2</para></a><para>3</para></chapter>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse(".//para").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<para>1</para><para>2</para><para>3</para>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_parent() {
        let (rest, tree) = xml_parser::document("<root><para /></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para/..").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<root><para></para></root>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_parent_attr_lang() {
        let (rest, tree) = xml_parser::document("<root lang='a'><para /></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("//para/../@lang").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("lang=\"a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_warning() {
        let (rest, tree) = xml_parser::document(
            "<root><para type='error' /><para type='warning' /><para type='normal' /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para[@type=\"warning\"]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\"></para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_warning_5() {
        let (rest, tree) =
            xml_parser::document("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para[@type=\"warning\"][5]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\">5</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_para_5_warning() {
        let (rest, tree) =
            xml_parser::document("<root><para type='warning'>1</para><para type='error' /><para type='warning'>2</para><para type='normal' /><para type='warning'>3</para><para type='warning'>4</para><para type='warning'>5</para></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/para[5][@type=\"warning\"]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<para type=\"warning\">3</para>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_chapter_intro() {
        let (rest, tree) =
            xml_parser::document("<root><chapter><title>Introduction</title></chapter><chapter><title>Second</title></chapter></root>").unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/chapter[title=\"Introduction\"]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<chapter><title>Introduction</title></chapter>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_abbreviated_chapter_title() {
        let (rest, tree) =
            xml_parser::document("<root><chapter></chapter><chapter><title /></chapter></root>")
                .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/chapter[title]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<chapter><title></title></chapter>", format!("{}", r));
    }

    #[test]
    fn test_eg_abbreviated_employee() {
        let (rest, tree) = xml_parser::document(
            "<root><employee secretary='a'/><employee secretary='a' assistant='b' /><employee a='b'/><employee assistant='b'/></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/employee[@secretary and @assistant]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<employee secretary=\"a\" assistant=\"b\"></employee>",
            format!("{}", r)
        );
    }

    #[test]
    fn test_eg_namespace_predicate() {
        let (rest, tree) = xml_parser::document(
            "<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/e2[namespace::a]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("<e2 xmlns:a=\"http://test/a\"></e2>", format!("{}", r));
    }

    #[test]
    fn test_eg_namespace_value() {
        let (rest, tree) = xml_parser::document(
            "<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/e2/namespace::a").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!("xmlns:a=\"http://test/a\"", format!("{}", r));
    }

    #[test]
    fn test_eg_namespace_default() {
        let (rest, tree) = xml_parser::document(
            "<root xmlns:b='http://test/b'><e2 xmlns='http://test/' /><e2 xmlns:a='http://test/a' /><e2 /></root>",
        )
        .unwrap();
        assert_eq!("", rest);
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());

        let (rest, expr) = expr::parse("root/e2[namespace::xml]").unwrap();
        assert_eq!("", rest);

        let r = eval::document(&expr, doc, &mut eval::model::Context::default()).unwrap();
        assert_eq!(
            "<e2 xmlns:a=\"http://test/a\"></e2><e2></e2>",
            format!("{}", r)
        );
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
