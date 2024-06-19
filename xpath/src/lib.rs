pub mod expr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eg_location_path_para() {
        let (rest, _) = expr::parse("child::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ns() {
        let (rest, _) = expr::parse("child::*").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_text() {
        let (rest, _) = expr::parse("child::text()").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_node() {
        let (rest, _) = expr::parse("child::node()").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_attr_name() {
        let (rest, _) = expr::parse("attribute::name").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_attr_ns() {
        let (rest, _) = expr::parse("attribute::*").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_descendant_para() {
        let (rest, _) = expr::parse("descendant::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ancestor_div() {
        let (rest, _) = expr::parse("ancestor::div").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ancestor_self_div() {
        let (rest, _) = expr::parse("ancestor-or-self::div").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_descendant_self_para() {
        let (rest, _) = expr::parse("descendant-or-self::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_self_para() {
        let (rest, _) = expr::parse("self::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_chapter_para() {
        let (rest, _) = expr::parse("child::chapter/descendant::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ns_para() {
        let (rest, _) = expr::parse("child::*/child::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_root() {
        let (rest, _) = expr::parse("/").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_root_descendant_para() {
        let (rest, _) = expr::parse("/descendant::para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_root_olist_item() {
        let (rest, _) = expr::parse("/descendant::olist/child::item").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_1() {
        let (rest, _) = expr::parse("child::para[position()=1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_last() {
        let (rest, _) = expr::parse("child::para[position()=last()]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_prelast() {
        let (rest, _) = expr::parse("child::para[position()=last()-1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_gt_1() {
        let (rest, _) = expr::parse("child::para[position()>1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_following_chapter_1() {
        let (rest, _) = expr::parse("following-sibling::chapter[position()=1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_preceding_chapter_1() {
        let (rest, _) = expr::parse("preceding-sibling::chapter[position()=1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_root_figure_42() {
        let (rest, _) = expr::parse("/descendant::figure[position()=42]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_root_chapter_5_section_2() {
        let (rest, _) =
            expr::parse("/child::doc/child::chapter[position()=5]/child::section[position()=2]")
                .unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_warning() {
        let (rest, _) = expr::parse("child::para[attribute::type=\"warning\"]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_warning_5() {
        let (rest, _) =
            expr::parse("child::para[attribute::type='warning'][position()=5]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_para_5_warning() {
        let (rest, _) =
            expr::parse("child::para[position()=5][attribute::type=\"warning\"]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_chapter_intro() {
        let (rest, _) = expr::parse("child::chapter[child::title='Introduction']").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_chapter_title() {
        let (rest, _) = expr::parse("child::chapter[child::title]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix() {
        let (rest, _) = expr::parse("child::*[self::chapter or self::appendix]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_location_path_ns_chapter_or_appendix_last() {
        let (rest, _) =
            expr::parse("child::*[self::chapter or self::appendix][position()=last()]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para() {
        let (rest, _) = expr::parse("para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_ns() {
        let (rest, _) = expr::parse("*").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_text() {
        let (rest, _) = expr::parse("text()").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_attr_name() {
        let (rest, _) = expr::parse("@name").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_attr_ns() {
        let (rest, _) = expr::parse("@*").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para_1() {
        let (rest, _) = expr::parse("para[1]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para_last() {
        let (rest, _) = expr::parse("para[last()]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_root_para() {
        let (rest, _) = expr::parse("/para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_root_chapter_5_section_2() {
        let (rest, _) = expr::parse("/doc/chapter[5]/section[2]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_chapter_para() {
        let (rest, _) = expr::parse("chapter//para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_root_descendant_para() {
        let (rest, _) = expr::parse("//para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_root_olist_item() {
        let (rest, _) = expr::parse("//olist/item").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_current() {
        let (rest, _) = expr::parse(".").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_current_descendant_para() {
        let (rest, _) = expr::parse(".//para").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_parent() {
        let (rest, _) = expr::parse("..").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_parent_attr_lang() {
        let (rest, _) = expr::parse("../@lang").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para_warning() {
        let (rest, _) = expr::parse("para[@type=\"warning\"]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para_warning_5() {
        let (rest, _) = expr::parse("para[@type=\"warning\"][5]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_para_5_warning() {
        let (rest, _) = expr::parse("para[5][@type=\"warning\"]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_chapter_intro() {
        let (rest, _) = expr::parse("chapter[title=\"Introduction\"]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_chapter_title() {
        let (rest, _) = expr::parse("chapter[title]").unwrap();
        assert_eq!("", rest);
    }

    #[test]
    fn test_eg_abbreviated_employee() {
        let (rest, _) = expr::parse("employee[@secretary and @assistant]").unwrap();
        assert_eq!("", rest);
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
