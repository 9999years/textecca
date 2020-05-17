use std::io::{self, Write};
use std::iter;

use html5ever::{
    interface::QualName,
    local_name, namespace_url, ns,
    serialize::{HtmlSerializer, SerializeOpts, Serializer, TraversalScope},
};

macro_rules! html_name {
    ($el_name:tt) => {
        QualName::new(None, ns!(html), local_name!($el_name))
    };
}

// pub fn to_html<'i>(tree: &ParseTree<'i>, writer: &mut impl Write) -> io::Result<()> {
//     let mut doc = HtmlSerializer::new(
//         writer,
//         SerializeOpts {
//             create_missing_parent: false,
//             scripting_enabled: false,
//             traversal_scope: TraversalScope::ChildrenOnly(None),
//         },
//     );
//     doc.write_doctype("html")?;
//     doc.write_text("\n")?;
//     let p_name = html_name!("p");
//     for paragraph in &tree.paragraphs {
//         doc.start_elem(p_name.clone(), iter::empty())?;
//         doc.write_text(paragraph.content.fragment())?;
//         doc.write_text("\n")?;
//     }
//     Ok(())
// }

// #[cfg(test)]
// mod test {
//     use indoc::indoc;
//     use pretty_assertions::assert_eq;

//     use super::*;

//     #[test]
//     fn to_html_trivial() {
//         let mut buf = Vec::new();
//         assert!(to_html(
//             &parse::<'_, ParseError>(indoc!(
//                 r#"
//                 First paragraph.

//                 Second paragraph.

//                 This & that.
//                 "#
//             ))
//             .unwrap()
//             .1,
//             &mut buf
//         )
//         .is_ok());

//         assert_eq!(
//             indoc!(
//                 r#"
//                 <!DOCTYPE html>
//                 <p>First paragraph.
//                 <p>Second paragraph.
//                 <p>This &amp; that.
//                 "#
//             ),
//             String::from_utf8(buf).unwrap()
//         );
//     }
// }
