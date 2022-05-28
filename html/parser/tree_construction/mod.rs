/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod rules {
    mod body;
    mod head;
    mod html;
    mod initial;
}

use std::{borrow::BorrowMut, ops::ControlFlow, sync::Arc};

use dom::node::{
    CommentNode, CreateElementOptions, Document, DocumentNode, Node,
    TextNode,
};
use html_elements::{
    interface::IsOneOfTagsInterface, tag_attributes, tag_names,
};
use infra::{
    namespace::Namespace, primitive::codepoint::CodePoint,
    structure::tree::TreeNode,
};
use macros::dd;

use crate::{
    state::{
        Entry, FormElementPointer, FramesetOkFlag, HeadElementPointer,
        InsertionMode, ListOfActiveFormattingElements, ScriptingFlag,
        StackOfOpenElements,
    },
    tokenization::{HTMLTagToken, HTMLToken, HTMLTokenizerState},
    HTMLParserFlag, HTMLParserState,
};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct HTMLTreeConstruction {
    document: DocumentNode,
    pub(crate) insertion_mode: InsertionMode,
    pub(crate) original_insertion_mode: InsertionMode,
    stack_of_template_insertion_modes: Vec<InsertionMode>,
    pub(crate) stack_of_open_elements: StackOfOpenElements,
    list_of_active_formatting_elements: ListOfActiveFormattingElements,
    foster_parenting: bool,
    scripting_flag: ScriptingFlag,
    pub(crate) frameset_ok_flag: FramesetOkFlag,
    parsing_fragment: bool,
    context_element: Option<TreeNode<Node>>,
    character_insertion_node: Option<TreeNode<Node>>,
    character_insertion_builder: String,
    head_element_pointer: Option<HeadElementPointer>,
    form_element_pointer: Option<FormElementPointer>,
    pending_table_character_tokens: Vec<HTMLToken>,
}

type HTMLTreeConstructionControlFlow =
    ControlFlow<HTMLParserFlag, HTMLParserState>;

struct AdjustedInsertionLocation {
    parent: Option<TreeNode<Node>>,
    insert_before_sibling: Option<TreeNode<Node>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLTreeConstruction {
    pub(crate) fn new(document: DocumentNode) -> Self {
        Self {
            document,
            ..Default::default()
        }
    }
}

impl HTMLTreeConstruction {
    pub(crate) fn dispatcher(
        &mut self,
        token: Option<HTMLToken>,
    ) -> ControlFlow<HTMLParserFlag, HTMLParserState> {
        match token {
            | None => ControlFlow::Break(HTMLParserFlag::Stop),

            | Some(token) if !self.use_foreign_process(&token) => self
                .process_using_the_rules_for(self.insertion_mode, token),

            // Traiter le jeton selon les règles indiquées dans la
            // section relative à l'analyse syntaxique des jetons dans
            // le contenu étranger.
            | Some(token) => {
                self.process_using_the_rules_for_foreign_content(token)
            }
        }
    }

    /// Le noeud courant ajusté est l'élément de contexte si l'analyseur a
    /// été créé dans le cadre de l'algorithme d'analyse des fragments HTML
    /// et que la pile d'éléments ouverts ne contient qu'un seul élément
    /// (cas du fragment) ; sinon, le noeud courant ajusté est le noeud
    /// courant.
    pub fn adjusted_current_node(&self) -> &TreeNode<Node> {
        if self.parsing_fragment && self.stack_of_open_elements.len() == 1
        {
            self.context_element.as_ref().expect("Context Element")
        } else {
            self.current_node().expect("Le noeud actuel")
        }
    }

    fn before_current_node(&self) -> Option<&TreeNode<Node>> {
        let size = self.stack_of_open_elements.len();
        if size == 0 || size == 1 {
            return None;
        }
        self.stack_of_open_elements.get(size - 2)
    }

    /// Le nœud actuel est le nœud le plus bas de cette pile d'éléments
    /// ouverts.
    fn current_node(&self) -> Option<&TreeNode<Node>> {
        self.stack_of_open_elements.current_node()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#using-the-rules-for>
    fn process_using_the_rules_for(
        &mut self,
        m: InsertionMode,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match dd!(&m) {
            | InsertionMode::Initial => {
                self.handle_initial_insertion_mode(token)
            }
            | InsertionMode::BeforeHTML => {
                self.handle_before_html_insertion_mode(token)
            }
            | InsertionMode::BeforeHead => {
                self.handle_before_head_insertion_mode(token)
            }
            | InsertionMode::InHead => {
                self.handle_in_head_insertion_mode(token)
            }
            | InsertionMode::InHeadNoscript => {
                self.handle_in_head_noscript_insertion_mode(token)
            }
            | InsertionMode::AfterHead => {
                self.handle_after_head_insertion_mode(token)
            }
            | InsertionMode::InBody => {
                self.handle_in_body_insertion_mode(token)
            }
            | InsertionMode::Text => {
                self.handle_text_insertion_mode(token)
            }
            | InsertionMode::InTable => {
                self.handle_in_table_insertion_mode(token)
            }
            | InsertionMode::InTableText => {
                self.handle_in_table_text_insertion_mode(token)
            }
            | InsertionMode::InCaption => {
                self.handle_in_caption_insertion_mode(token)
            }
            | InsertionMode::InColumnGroup => {
                self.handle_in_column_group_insertion_mode(token)
            }
            | InsertionMode::InTableBody => {
                self.handle_in_table_body_insertion_mode(token)
            }
            | InsertionMode::InRow => {
                self.handle_in_row_insertion_mode(token)
            }
            | InsertionMode::InCell => {
                self.handle_in_cell_insertion_mode(token)
            }
            | InsertionMode::InSelect => {
                self.handle_in_select_insertion_mode(token)
            }
            | InsertionMode::InSelectInTable => {
                self.handle_in_select_in_table_insertion_mode(token)
            }
            | InsertionMode::InTemplate => {
                self.handle_in_template_insertion_mode(token)
            }
            | InsertionMode::AfterBody => {
                self.handle_after_body_insertion_mode(token)
            }
            | InsertionMode::InFrameset => {
                self.handle_in_frameset_insertion_mode(token)
            }
            | InsertionMode::AfterFrameset => {
                self.handle_after_frameset_insertion_mode(token)
            }
            | InsertionMode::AfterAfterBody => {
                self.handle_after_after_body_insertion_mode(token)
            }
            | InsertionMode::AfterAfterFrameset => {
                self.handle_after_after_frameset_insertion_mode(token)
            }
        }
    }

    fn process_using_the_rules_for_foreign_content(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Insérer un caractère U+FFFD REPLACEMENT
            // CHARACTER.
            | HTMLToken::Character('\0') => {
                self.parse_error(&token);
                self.insert_character(char::REPLACEMENT_CHARACTER);
            }

            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // Any other character token
            //
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) => {
                self.insert_character(ch);
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // A comment token
            //
            // Insérer le commentaire.
            | HTMLToken::Comment(comment) => self.insert_comment(comment),

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
            }

            // A start tag whose tag name is one of:
            //   - "b", "big", "blockquote", "body", "br", "center",
            //     "code", "dd", "div", "dl", "dt", "em", "embed", "h1",
            //     "h2", "h3", "h4", "h5", "h6", "head", "hr", "i", "img",
            //     "li", "listing", "menu", "meta", "nobr", "ol", "p",
            //     "pre", "ruby", "s", "small", "span", "strong", "strike",
            //     "sub", "sup", "table", "tt", "u", "ul", "var"
            // A start tag whose tag name is "font", if the token has any
            // attributes named "color", "face", or "size"
            // An end tag whose tag name is "br", "p"
            //
            // Erreur d'analyse.
            // Si le nœud actuel n'est pas un point d'intégration de texte
            // MathML, un point d'intégration HTML ou un élément de
            // l'espace de noms HTML, il faut extraire les éléments de la
            // pile des éléments ouverts.
            // Retraiter le jeton selon les règles données dans la section
            // correspondant au mode d'insertion actuel dans le contenu
            // HTML.
            #[allow(deprecated)]
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name, is_end, ..
                },
            ) if (!is_end
                && (name.is_one_of([
                    tag_names::b,
                    tag_names::big,
                    tag_names::blockquote,
                    tag_names::body,
                    tag_names::br,
                    tag_names::center,
                    tag_names::code,
                    tag_names::dd,
                    tag_names::div,
                    tag_names::dl,
                    tag_names::dt,
                    tag_names::em,
                    tag_names::embed,
                    tag_names::h1,
                    tag_names::h2,
                    tag_names::h3,
                    tag_names::h4,
                    tag_names::h5,
                    tag_names::h6,
                    tag_names::head,
                    tag_names::hr,
                    tag_names::i,
                    tag_names::img,
                    tag_names::li,
                    tag_names::listing,
                    tag_names::menu,
                    tag_names::meta,
                    tag_names::nobr,
                    tag_names::ol,
                    tag_names::p,
                    tag_names::pre,
                    tag_names::ruby,
                    tag_names::s,
                    tag_names::small,
                    tag_names::span,
                    tag_names::strong,
                    tag_names::strike,
                    tag_names::sub,
                    tag_names::sup,
                    tag_names::table,
                    tag_names::tt,
                    tag_names::u,
                    tag_names::ul,
                    tag_names::var,
                ]) || tag_names::font == name
                    && tag_token.has_attributes([
                        tag_attributes::color,
                        tag_attributes::face,
                        tag_attributes::size,
                    ])))
                || is_end
                    && name.is_one_of([tag_names::br, tag_names::p]) =>
            {
                self.parse_error(&token);

                let maybe_cnode = self.current_node().cloned();

                if maybe_cnode.is_none() {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                while let Some(cnode) = maybe_cnode.as_ref() {
                    if !cnode.is_mathml_text_integration_point()
                        && !cnode.is_html_text_integration_point()
                        && !cnode.isin_html_namespace()
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Any other start tag
            //
            // Si le nœud courant ajusté est un élément de l'espace de noms
            // MathML, nous devons ajuster les attributs MathML pour le
            // jeton. (Cela corrige le cas des attributs MathML qui ne sont
            // pas tous en minuscules).
            // Si le nœud courant ajusté est un élément de l'espace de noms
            // SVG, et que le nom de la balise du jeton est l'un de ceux de
            // la première colonne du tableau suivant, nous devons changer
            // le nom de la balise par le nom donné dans la cellule
            // correspondante de la deuxième colonne. (Ceci règle le cas
            // des éléments SVG qui ne sont pas tous en  minuscules).
            // Si le noeud courant ajusté est un élément dans l'espace de
            // nom SVG, ajuster les attributs SVG pour le jeton. (Cela
            // corrige le cas des attributs SVG qui ne sont pas tous en
            // minuscules).
            // Ajuster les attributs étrangers pour le jeton. (Cela corrige
            // l'utilisation d'attributs espacés par des noms, en
            // particulier XLink dans SVG).
            // Insérer un élément étranger pour le jeton, dans le même
            // espace de nom que le nœud courant ajusté.
            // Si le jeton a son drapeau de self-closing activé,
            // nous devons exécuter les étapes appropriées de la liste
            // suivante :
            //   - Si le nom de balise du jeton est "script", et que le
            //     nouveau nœud courant se trouve dans l'espace de noms
            //     SVG.
            //     - Accuser réception du drapeau de fermeture automatique
            //       du jeton, puis agir comme décrit dans les étapes pour
            //       une balise de fin "script" ci-dessous.
            //   - Sinon
            //     - Retirer le nœud actuel de la pile des éléments ouverts
            //       et reconnaître le drapeau de self-closing du jeton.
            | HTMLToken::Tag(
                mut tag_token @ HTMLTagToken {
                    is_end: false,
                    self_closing_flag,
                    ..
                },
            ) => {
                let adjusted_current_node =
                    self.adjusted_current_node().element_ref();

                let maybe_acn_namespace =
                    adjusted_current_node.namespace();

                if let Some(Namespace::MathML) = maybe_acn_namespace {
                    self.adjust_mathml_attributes(&mut tag_token);
                } else if let Some(Namespace::SVG) = maybe_acn_namespace {
                    self.adjust_svg_tag_name(&mut tag_token);
                    self.adjust_svg_attributes(&mut tag_token);
                }

                self.adjust_foreign_attributes(&mut tag_token);

                self.insert_foreign_element(
                    &tag_token,
                    maybe_acn_namespace
                        .expect("Devrait être un espace de noms valide"),
                );

                if !self_closing_flag {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let cnode = self.current_node().expect("Le noeud actuel.");
                if tag_names::script == tag_token.name
                    && cnode.element_ref().isin_svg_namespace()
                {
                    tag_token.set_acknowledge_self_closing_flag();
                    return self.process_using_the_rules_for(
                        self.insertion_mode,
                        HTMLToken::Tag(tag_token),
                    );
                } else {
                    self.stack_of_open_elements.pop();
                }
            }

            | _ => todo!(),
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    /// Lorsque chaque jeton est émis par le tokenizer, l'agent
    /// utilisateur doit suivre les étapes appropriées de la
    /// liste suivante, connue sous le nom de dispatcher de
    /// construction d'arbre :
    ///    - Si la pile d'éléments ouverts est vide
    ///    - Si le nœud courant ajusté est un élément dans l'espace de nom
    ///      HTML
    ///    - Si le nœud courant ajusté est un point d'intégration de texte
    ///      MathML et que le jeton est une balise de début dont le nom de
    ///      balise n'est ni "mglyph" ni "malignmark"
    ///    - Si le nœud courant ajusté est un point d'intégration de texte
    ///      MathML et que le jeton est un jeton de caractère
    ///    - Si le nœud courant ajusté est un élément MathML annotation-xml
    ///      et que le jeton est une balise de départ dont le nom de balise
    ///      est "svg"
    ///    - Si le nœud courant ajusté est un point d'intégration HTML et
    ///      que le jeton est une balise de départ
    ///    - Si le nœud courant ajusté est un point d'intégration HTML et
    ///      que le jeton est un jeton de caractère
    ///    - Si le jeton est un jeton de fin de fichier
    ///
    /// Traiter le jeton selon les règles données dans la
    /// section correspondant au mode d'insertion actuel dans le
    /// contenu HTML.
    fn use_foreign_process(&self, token: &HTMLToken) -> bool {
        !(self.stack_of_open_elements.is_empty()
            || self.adjusted_current_node().isin_html_namespace()
            || self
                .adjusted_current_node()
                .is_mathml_text_integration_point()
                && token.is_start_tag()
                && !token
                    .as_tag()
                    .name
                    .is_one_of([tag_names::mglyph, tag_names::malignmark])
            || self
                .adjusted_current_node()
                .is_mathml_text_integration_point()
                && token.is_character()
            || self.adjusted_current_node().element_ref().tag_name()
                == tag_names::annotationXml
                && token.is_start_tag()
                && tag_names::svg == token.as_tag().name
            || (self
                .adjusted_current_node()
                .is_html_text_integration_point()
                && (token.is_start_tag() || token.is_character()))
            || token.is_eof())
    }
}

impl HTMLTreeConstruction {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes>
    fn adjust_foreign_attributes(&mut self, tag_token: &mut HTMLTagToken) {
        [
            ("xlink:actuate", "xlink", "actuate", Namespace::XLink),
            ("xlink:arcrole", "xlink", "arcrole", Namespace::XLink),
            ("xlink:href", "xlink", "href", Namespace::XLink),
            ("xlink:role", "xlink", "role", Namespace::XLink),
            ("xlink:show", "xlink", "show", Namespace::XLink),
            ("xlink:title", "xlink", "title", Namespace::XLink),
            ("xlink:type", "xlink", "type", Namespace::XLink),
            ("xml:lang", "xml", "lang", Namespace::XML),
            ("xml:space", "xml", "space", Namespace::XML),
            ("xmlns", "", "xmlns", Namespace::XMLNS),
            ("xmlns:xlink", "xmlns", "xlink", Namespace::XMLNS),
        ]
        .into_iter()
        .for_each(|(old_name, prefix, local_name, ns)| {
            tag_token.adjust_foreign_attribute(
                old_name, prefix, local_name, ns,
            );
        });
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-mathml-attributes>
    fn adjust_mathml_attributes(&mut self, tag_token: &mut HTMLTagToken) {
        [("definitionurl", "definitionURL")].into_iter().for_each(
            |(old_name, new_name)| {
                tag_token.adjust_attribute_name(old_name, new_name);
            },
        );
    }

    fn adjust_svg_tag_name(&mut self, tag_token: &mut HTMLTagToken) {
        [
            ("altglyph", "altGlyph"),
            ("altglyphdef", "altGlyphDef"),
            ("altglyphitem", "altGlyphItem"),
            ("animatecolor", "animateColor"),
            ("animatemotion", "animateMotion"),
            ("animatetransform", "animateTransform"),
            ("clippath", "clipPath"),
            ("feblend", "feBlend"),
            ("fecolormatrix", "feColorMatrix"),
            ("fecomponenttransfer", "feComponentTransfer"),
            ("fecomposite", "feComposite"),
            ("feconvolvematrix", "feConvolveMatrix"),
            ("fediffuselighting", "feDiffuseLighting"),
            ("fedisplacementmap", "feDisplacementMap"),
            ("fedistantlight", "feDistantLight"),
            ("fedropshadow", "feDropShadow"),
            ("feflood", "feFlood"),
            ("fefunca", "feFuncA"),
            ("fefuncb", "feFuncB"),
            ("fefuncg", "feFuncG"),
            ("fefuncr", "feFuncR"),
            ("fegaussianblur", "feGaussianBlur"),
            ("feimage", "feImage"),
            ("femerge", "feMerge"),
            ("femergenode", "feMergeNode"),
            ("femorphology", "feMorphology"),
            ("feoffset", "feOffset"),
            ("fepointlight", "fePointLight"),
            ("fespecularlighting", "feSpecularLighting"),
            ("fespotlight", "feSpotLight"),
            ("fetile", "feTile"),
            ("feturbulence", "feTurbulence"),
            ("foreignobject", "foreignObject"),
            ("glyphref", "glyphRef"),
            ("lineargradient", "linearGradient"),
            ("radialgradient", "radialGradient"),
            ("textpath", "textPath"),
        ]
        .into_iter()
        .for_each(|(old_name, new_name)| {
            tag_token.adjust_tag_name(old_name, new_name);
        });
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-svg-attributes>
    fn adjust_svg_attributes(&mut self, tag_token: &mut HTMLTagToken) {
        [
            ("attributename", "attributeName"),
            ("attributetype", "attributeType"),
            ("basefrequency", "baseFrequency"),
            ("baseprofile", "baseProfile"),
            ("calcmode", "calcMode"),
            ("clippathunits", "clipPathUnits"),
            ("diffuseconstant", "diffuseConstant"),
            ("edgemode", "edgeMode"),
            ("filterunits", "filterUnits"),
            ("glyphref", "glyphRef"),
            ("gradienttransform", "gradientTransform"),
            ("gradientunits", "gradientUnits"),
            ("kernelmatrix", "kernelMatrix"),
            ("kernelunitlength", "kernelUnitLength"),
            ("keypoints", "keyPoints"),
            ("keysplines", "keySplines"),
            ("keytimes", "keyTimes"),
            ("lengthadjust", "lengthAdjust"),
            ("limitingconeangle", "limitingConeAngle"),
            ("markerheight", "markerHeight"),
            ("markerunits", "markerUnits"),
            ("markerwidth", "markerWidth"),
            ("maskcontentunits", "maskContentUnits"),
            ("maskunits", "maskUnits"),
            ("numoctaves", "numOctaves"),
            ("pathlength", "pathLength"),
            ("patterncontentunits", "patternContentUnits"),
            ("patterntransform", "patternTransform"),
            ("patternunits", "patternUnits"),
            ("pointsatx", "pointsAtX"),
            ("pointsaty", "pointsAtY"),
            ("pointsatz", "pointsAtZ"),
            ("preservealpha", "preserveAlpha"),
            ("preserveaspectratio", "preserveAspectRatio"),
            ("primitiveunits", "primitiveUnits"),
            ("refx", "refX"),
            ("refy", "refY"),
            ("repeatcount", "repeatCount"),
            ("repeatdur", "repeatDur"),
            ("requiredextensions", "requiredExtensions"),
            ("requiredfeatures", "requiredFeatures"),
            ("specularconstant", "specularConstant"),
            ("specularexponent", "specularExponent"),
            ("spreadmethod", "spreadMethod"),
            ("startoffset", "startOffset"),
            ("stddeviation", "stdDeviation"),
            ("stitchtiles", "stitchTiles"),
            ("surfacescale", "surfaceScale"),
            ("systemlanguage", "systemLanguage"),
            ("tablevalues", "tableValues"),
            ("targetx", "targetX"),
            ("targety", "targetY"),
            ("textlength", "textLength"),
            ("viewbox", "viewBox"),
            ("viewtarget", "viewTarget"),
            ("xchannelselector", "xChannelSelector"),
            ("ychannelselector", "yChannelSelector"),
            ("zoomandpan", "zoomAndPan"),
        ]
        .into_iter()
        .for_each(|(old_name, new_name)| {
            tag_token.adjust_attribute_name(old_name, new_name);
        });
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    fn create_element_for(
        &mut self,
        token: &HTMLTagToken,
        namespace: Namespace,
        intended_parent: Option<&TreeNode<Node>>,
    ) -> Option<TreeNode<Node>> {
        let HTMLTagToken {
            name: local_name,
            attributes,
            ..
        } = token;

        let document = intended_parent.unwrap_or(&self.document);

        let maybe_element = Document::create_element(
            local_name,
            Some(CreateElementOptions {
                is: None,
                namespace: Some(namespace),
            }),
        );

        if let Ok(element) = maybe_element.as_ref() {
            element.set_document(document);

            attributes.iter().for_each(|attribute| {
                element
                    .element_ref()
                    .set_attribute(&attribute.name, &attribute.value);
            });
        }

        maybe_element.ok()
    }

    /// L'endroit approprié pour insérer un nœud, en utilisant
    /// éventuellement une cible prioritaire particulière, est la position
    /// dans un élément renvoyé par l'exécution des étapes suivantes :
    ///
    /// 1. Si une cible prioritaire a été spécifiée, alors la cible est la
    /// cible prioritaire.
    ///
    /// 2. Déterminer l'emplacement d'insertion ajusté en utilisant les
    /// premières étapes de correspondance de la liste suivante :
    ///
    ///    2.1. Si le `foster parenting` est activée et que la cible est
    /// un élément table, tbody, tfoot, thead ou tr.
    ///
    ///    Note: Le `foster parenting` se produit lorsque le contenu est
    /// mal intégré dans les table's.
    ///
    ///      2.1.1. Le dernier template est le dernier élément template
    /// dans la pile d'éléments ouverts, s'il y en a.
    ///
    ///      2.1.2. Le dernier table est le dernier élément table dans la
    /// pile des éléments ouverts, s'il y en a.
    ///
    ///      2.1.3. S'il y en a un dernier template et qu'il n'y a pas de
    /// dernière table, ou s'il y en a une, mais que le dernier template
    /// est plus bas (plus récemment ajouté) que la dernière table dans la
    /// pile des éléments ouverts, alors : nous devons laisser
    /// l'emplacement d'insertion ajusté à l'intérieur du contenu du
    /// template du dernier template, après son dernier enfant (s'il y
    /// en a), et abandonner ces étapes.
    ///
    ///      2.1.4. S'il n'y a pas de dernier table, alors l'emplacement
    /// d'insertion ajusté se trouve à l'intérieur du premier élément de la
    /// pile d'éléments ouverts (l'élément html), après son dernier enfant
    /// (s'il y en a un), et on abandonne ces étapes. (cas d'un fragment)
    ///
    ///      2.1.5. Si la dernière table a un noeud parent, alors
    /// l'emplacement d'insertion ajusté sera à l'intérieur du noeud parent
    /// de la dernière table, immédiatement avant la dernière
    ///        table, et annulera ces étapes.
    ///
    ///      2.1.6. Laisser "l'élément précédent" être l'élément
    /// directement au-dessus de la dernière table dans la pile des
    /// éléments ouverts.
    ///
    ///      2.1.7. Que l'emplacement d'insertion ajusté soit à
    /// l'intérieur de l'élément précédent, après son dernier enfant (le
    /// cas échéant).
    ///
    ///    Note: Ces étapes sont nécessaires en partie parce qu'il est
    /// possible que des éléments, en particulier l'élément table dans ce
    /// cas, aient été déplacés par un script dans le DOM, ou même
    /// entièrement retirés du DOM, après que l'élément ait été inséré par
    /// l'analyseur.
    ///
    ///    2.2. Sinon : l'emplacement d'insertion ajusté doit être à
    /// l'intérieur de la cible, après son dernier enfant (s'il y en a).
    ///
    /// 3. Si l'emplacement d'insertion ajusté se trouve à l'intérieur d'un
    /// élément template, il doit plutôt se trouver à l'intérieur du
    /// contenu template de l'élément template, après son dernier enfant
    /// (s'il y en a).
    ///
    /// 4. Retourner l'emplacement d'insertion ajusté.
    fn find_appropriate_place_for_inserting_node(
        &self,
        override_target: Option<&TreeNode<Node>>,
    ) -> AdjustedInsertionLocation {
        let maybe_target = override_target.or_else(|| self.current_node());

        let mut adjusted_insertion_location = AdjustedInsertionLocation {
            insert_before_sibling: None,
            parent: None,
        };

        if self.foster_parenting
            && [
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
            ]
            .into_iter()
            .any(|local_name| {
                if let Some(target) = maybe_target {
                    local_name == target.element_ref().local_name()
                } else {
                    false
                }
            })
        {
            let last_template = self
                .stack_of_open_elements
                .get_last_element_with_tag_name(tag_names::template);
            let last_table = self
                .stack_of_open_elements
                .get_last_element_with_tag_name(tag_names::table);

            if let Some((template_index, template)) = last_template {
                fn return_adjusted_insertion_location(
                    template: &TreeNode<Node>,
                ) -> AdjustedInsertionLocation {
                    let tc = template
                        .element_ref()
                        .content()
                        .map(|t| t.to_owned());
                    AdjustedInsertionLocation {
                        parent: tc,
                        insert_before_sibling: None,
                    }
                }

                if last_table.is_none() {
                    return return_adjusted_insertion_location(template);
                }

                if let Some((table_index, _)) = last_table {
                    if template_index > table_index {
                        return return_adjusted_insertion_location(
                            template,
                        );
                    }
                }
            }

            if last_table.is_none() {
                assert!(self.parsing_fragment);

                return AdjustedInsertionLocation {
                    parent: self.stack_of_open_elements.first().cloned(),
                    insert_before_sibling: None,
                };
            }

            if let Some((table_index, table)) = last_table {
                let parent = table.parent_node();
                if let Some(node) = parent {
                    adjusted_insertion_location.parent = node.into();
                    adjusted_insertion_location
                        .insert_before_sibling
                        .replace(table.to_owned());
                } else {
                    let previous_element = self
                        .stack_of_open_elements
                        .element_immediately_above(table_index)
                        .map(|(_, p)| p.to_owned());
                    adjusted_insertion_location.parent = previous_element;
                }
            }
        } else {
            adjusted_insertion_location = AdjustedInsertionLocation {
                parent: maybe_target.cloned(),
                insert_before_sibling: None,
            };
        }

        adjusted_insertion_location
    }

    fn find_character_insertion_node(&self) -> Option<TreeNode<Node>> {
        let adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let parent = adjusted_insertion_location.parent?;

        if parent.is_document() {
            return None;
        }

        let is_text = parent
            .get_last_child()
            .as_ref()
            .filter(|last_child| last_child.is_text())
            .cloned();

        if is_text.is_some() {
            return is_text;
        }

        let new_text_node = TextNode::new(&self.document, String::new());
        parent.append_child(new_text_node.to_owned());
        Some(new_text_node.to_owned())
    }

    fn flush_character_insertions(&mut self) {
        if self.character_insertion_builder.is_empty() {
            return;
        }

        if let Some(character_insertion_node) =
            self.character_insertion_node.as_ref()
        {
            character_insertion_node
                .set_data(&self.character_insertion_builder);
            self.character_insertion_builder.clear();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags>
    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_with_predicate(|name| {
            !name.is_empty()
        });
    }

    #[allow(deprecated)]
    fn generate_implied_end_tags_with_predicate(
        &mut self,
        predicate: impl Fn(&str) -> bool,
    ) {
        while let Some(cnode) = self.current_node() {
            let element = cnode.element_ref();
            let name = element.local_name();
            if predicate(&name)
                && name.is_one_of([
                    tag_names::dd,
                    tag_names::dt,
                    tag_names::li,
                    tag_names::optgroup,
                    tag_names::option,
                    tag_names::p,
                    tag_names::rb,
                    tag_names::rp,
                    tag_names::rt,
                    tag_names::rtc,
                ])
            {
                self.stack_of_open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn generate_implied_end_tags_except_for(
        &mut self,
        exception: tag_names,
    ) {
        self.generate_implied_end_tags_with_predicate(|name| {
            exception != name
        });
    }

    /// Lorsque les étapes ci-dessous exigent que l'UA génère de manière
    /// exhaustive toutes les balises de fin implicites, alors, si le noeud
    /// actuel est un élément caption, un élément colgroup, un élément dd,
    /// un élément dt, un élément "li", un élément optgroup, un élément
    /// option, un élément p, un élément rb, un élément rp, un élément rt,
    /// un élément rtc, un élément tbody, un élément td, un élément tfoot,
    /// un élément th, un élément thead ou un élément tr, l'UA doit retirer
    /// le noeud actuel de la pile des éléments ouverts.
    #[allow(deprecated)]
    fn generate_all_implied_end_tags_thoroughly(&mut self) {
        while let Some(cnode) = self.current_node() {
            if cnode.element_ref().local_name().is_one_of([
                tag_names::caption,
                tag_names::colgroup,
                tag_names::dd,
                tag_names::dt,
                tag_names::li,
                tag_names::optgroup,
                tag_names::option,
                tag_names::p,
                tag_names::rb,
                tag_names::rp,
                tag_names::rt,
                tag_names::rtc,
                tag_names::tbody,
                tag_names::td,
                tag_names::tfoot,
                tag_names::th,
                tag_names::thead,
                tag_names::tr,
            ]) {
                self.stack_of_open_elements.pop();
            } else {
                break;
            }
        }
    }

    // <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    fn insert_character(&mut self, ch: CodePoint) {
        let maybe_node = self.find_character_insertion_node();

        if let (Some(a), Some(b)) =
            (maybe_node.as_ref(), self.character_insertion_node.as_ref())
        {
            if a == b {
                self.character_insertion_builder.push(ch);
                return;
            }
        }

        if self.character_insertion_node.is_none() {
            self.character_insertion_node = maybe_node;
            self.character_insertion_builder.push(ch);
            return;
        }

        self.flush_character_insertions();
        self.character_insertion_node = maybe_node;
        self.character_insertion_builder.push(ch);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment(&self, comment: String) {
        let mut adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let comment = CommentNode::new(&self.document, comment);

        if let Some(ref mut parent) = adjusted_insertion_location.parent {
            parent.insert_before(
                comment.to_owned(),
                adjusted_insertion_location.insert_before_sibling.as_ref(),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    fn insert_html_element(
        &mut self,
        token: &HTMLTagToken,
    ) -> Option<TreeNode<Node>> {
        self.insert_foreign_element(token, Namespace::HTML)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    fn insert_foreign_element(
        &mut self,
        token: &HTMLTagToken,
        namespace: Namespace,
    ) -> Option<TreeNode<Node>> {
        let adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let maybe_element =
            self.create_element_for(token, namespace, None);

        if let Some(element) = maybe_element.as_ref() {
            self.stack_of_open_elements.put(element.to_owned());
            if let Some(parent) = adjusted_insertion_location.parent {
                if let Some(sibling) = adjusted_insertion_location
                    .insert_before_sibling
                    .as_ref()
                {
                    parent
                        .insert_before(element.to_owned(), Some(sibling));
                    return maybe_element;
                }

                parent.append_child(element.to_owned());
            }
        }

        maybe_element
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-errors>
    fn parse_error(&self, token: &HTMLToken) {
        match token {
            | HTMLToken::Tag(HTMLTagToken { name, is_end, .. }) => {
                if *is_end {
                    log::error!("Balise de fin inattendue: {name}");
                } else {
                    log::error!("Balise de début inattendue: {name}");
                }
            }
            | HTMLToken::DOCTYPE(_) => log::error!("DOCTYPE inattendu"),
            | HTMLToken::Comment(_) => {
                log::error!("Commentaire inattendu")
            }
            | HTMLToken::Character(_) => {
                log::error!("Caractère inattendu")
            }
            | HTMLToken::EOF => log::error!("End Of File: inattendu"),
        }
    }

    /// L'algorithme générique d'analyse syntaxique des éléments de
    /// texte brut et l'algorithme générique d'analyse syntaxique
    /// des éléments RCDATA comportent les étapes suivantes. Ces
    /// algorithmes sont toujours invoqués en réponse à un jeton de
    /// balise de début.
    ///
    ///   1. Insertion d'un élément HTML pour le jeton.
    ///   2. Si l'algorithme invoqué est l'algorithme générique
    /// d'analyse syntaxique des éléments de texte brut, faire
    /// passer le tokenizer à l'état RAWTEXT ; sinon, si
    /// l'algorithme invoqué est l'algorithme générique d'analyse
    /// syntaxique des éléments RCDATA, faire passer le tokenizer à
    /// l'état RCDATA.
    ///   3. Le mode d'insertion d'origine est le mode d'insertion
    /// actuel.
    ///   4. Ensuite, faire passer le mode d'insertion à "text".
    fn parse_generic_element(
        &mut self,
        tag_token: &HTMLTagToken,
        state: HTMLTokenizerState,
    ) -> HTMLTreeConstructionControlFlow {
        self.insert_html_element(tag_token);
        self.original_insertion_mode.switch_to(self.insertion_mode);
        self.insertion_mode.switch_to(InsertionMode::Text);
        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::SwitchTo(state.to_string()),
        )
    }

    /// Lorsque les étapes ci-dessous exigent que l'UA reconstruise les
    /// éléments de mise en forme actifs, l'UA doit effectuer les étapes
    /// suivantes :
    ///   1. S'il n'y a aucune entrée dans la liste des éléments de
    /// formatage actifs, alors il n'y a rien à reconstruire ; stopper
    /// l'algorithme.
    ///   2. Si la dernière entrée (la plus récemment ajoutée) dans la
    /// liste des éléments de mise en forme actifs est un marqueur, ou si
    /// c'est un élément qui se trouve dans la pile des éléments ouverts,
    /// alors il n'y a rien à reconstruire ; stopper l'algorithme.
    ///   3. Laisser entry être le dernier élément (le plus récemment
    /// ajouté) dans la liste des éléments de formatage actifs.
    ///   4. `rewind` : s'il n'y a aucune entrée avant l'entrée dans la
    /// liste des éléments de mise en forme actifs, nous devons passer à
    /// l'étape intitulée `create`.
    ///   5. Laisser entry être l'entrée antérieure à entry dans la liste
    /// des éléments de mise en forme actifs.
    ///   6. Si l'entrée n'est ni un marqueur ni un élément qui se trouve
    /// également dans la pile des éléments ouverts, nous devons passer à
    /// l'étape intitulée `rewind`.
    ///   7. `advance` : l'entrée est l'élément qui suit l'entrée dans la
    /// liste des éléments de mise en forme actifs.
    ///   8. `create` : insérer un élément HTML pour le jeton pour lequel
    /// l'entrée de l'élément a été créée, pour obtenir un nouvel élément.
    ///   9. Remplacer l'entrée pour l'élément dans la liste par une entrée
    /// pour le nouvel élément.
    ///   10. Si l'entrée pour le nouvel élément dans la liste des éléments
    /// de formatage actifs n'est pas la dernière entrée de la liste,
    /// nous devons revenir à l'étape intitulée Avancer.
    ///
    /// Cela a pour effet de rouvrir tous les éléments de mise en forme qui
    /// ont été ouverts dans le body, cell ou caption courant (selon le
    /// plus jeune) et qui n'ont pas été explicitement fermés.
    ///
    /// Note: La liste des éléments de formatage actifs est toujours
    /// constituée d'éléments dans l'ordre chronologique, l'élément le
    /// moins récemment ajouté étant le premier et l'élément le plus
    /// récemment ajouté le dernier (sauf pendant l'exécution des étapes 7
    /// à 10 de l'algorithme ci-dessus, bien sûr).
    fn reconstruct_active_formatting_elements(&mut self) {
        if self.list_of_active_formatting_elements.is_empty() {
            return;
        }

        let size = self.list_of_active_formatting_elements.len();

        let (mut entry, mut idx) = if let Some(last) =
            self.list_of_active_formatting_elements.last_mut()
        {
            if last.is_marker() {
                return;
            }

            if let Some(node) = last.element() {
                if self.stack_of_open_elements.contains(node) {
                    return;
                }
            }

            (last, size - 1)
        } else {
            log::info!("Ne devrait jamais tomber dans cette condition.");
            return;
        };

        'main: loop {
            // Rewind
            'rewind: loop {
                if idx == 0 {
                    break 'rewind /* continue in 'create */;
                }

                idx -= 1;
                entry = unsafe {
                    self.list_of_active_formatting_elements
                        .get_unchecked_mut(idx)
                }
                .borrow_mut();

                if !entry.is_marker()
                    && !self
                        .stack_of_open_elements
                        .contains(entry.element_unchecked())
                {
                    continue 'rewind;
                }
            }

            'create: loop {
                let element = self
                    .list_of_active_formatting_elements
                    .get(idx)
                    .and_then(|entry| entry.element())
                    .unwrap_or_else(|| {
                        panic!("L'élément à index {}", idx)
                    });

                let element = {
                    let tag_token = HTMLTagToken::start()
                        .with_name(element.element_ref().local_name());
                    self.insert_html_element(&tag_token)
                }
                .unwrap();

                self.list_of_active_formatting_elements
                    .get(idx)
                    .replace(&Entry::Element(element));

                if idx == size - 1 {
                    break 'create; /* continue in 'advance */
                }
            }

            'advance: loop {
                idx += 1;
                entry = unsafe {
                    self.list_of_active_formatting_elements
                        .get_unchecked_mut(idx)
                };
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately>
    fn reset_insertion_mode_appropriately(&mut self) {
        for (index, node) in
            self.stack_of_open_elements.iter().enumerate().rev()
        {
            let last = index == 0;

            let node = if last && self.parsing_fragment {
                self.context_element.to_owned().unwrap()
            } else {
                node.to_owned()
            };

            let element = node.element_ref();
            let element_tag_name = element.tag_name();

            match element_tag_name {
                | tag_names::select => {
                    for ancestor in
                        self.stack_of_open_elements[0..index].iter().rev()
                    {
                        let ancestor_tag_name =
                            ancestor.element_ref().tag_name();
                        match ancestor_tag_name {
                            | tag_names::template => break,
                            | tag_names::table => {
                                self.insertion_mode.switch_to(
                                    InsertionMode::InSelectInTable,
                                );
                                return;
                            }
                            | _ => {}
                        }
                    }

                    self.insertion_mode.switch_to(InsertionMode::InSelect);
                    return;
                }

                | tag if tag.is_one_of([tag_names::td, tag_names::th])
                    && !last =>
                {
                    self.insertion_mode.switch_to(InsertionMode::InCell);
                    return;
                }

                | tag_names::tr => {
                    self.insertion_mode.switch_to(InsertionMode::InRow);
                    return;
                }

                | tag if tag.is_one_of([
                    tag_names::tbody,
                    tag_names::thead,
                    tag_names::tfoot,
                ]) && !last =>
                {
                    self.insertion_mode
                        .switch_to(InsertionMode::InTableBody);
                    return;
                }

                | tag_names::caption => {
                    self.insertion_mode
                        .switch_to(InsertionMode::InCaption);
                    return;
                }

                | tag_names::colgroup => {
                    self.insertion_mode
                        .switch_to(InsertionMode::InColumnGroup);
                    return;
                }

                | tag_names::table => {
                    self.insertion_mode.switch_to(InsertionMode::InTable);
                    return;
                }

                | tag_names::template => {
                    let mode = *self
                        .stack_of_template_insertion_modes
                        .last()
                        .expect("Le dernier mode d'insertion de la pile template.");
                    self.insertion_mode.switch_to(mode);
                    return;
                }

                | tag_names::head if !last => {
                    self.insertion_mode.switch_to(InsertionMode::InHead);
                    return;
                }

                | tag_names::body => {
                    self.insertion_mode.switch_to(InsertionMode::InBody);
                    return;
                }

                #[allow(deprecated)]
                | tag_names::frameset => {
                    self.insertion_mode
                        .switch_to(InsertionMode::InFrameset);
                    return;
                }

                | tag_names::html => {
                    if self.head_element_pointer.is_none() {
                        self.insertion_mode
                            .switch_to(InsertionMode::BeforeHead);
                        return;
                    }

                    self.insertion_mode
                        .switch_to(InsertionMode::AfterHead);
                    return;
                }

                | _ if last => break,

                | _ => {}
            }
        }

        self.insertion_mode.switch_to(InsertionMode::InBody);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm
    fn run_adoption_agency_algorithm(
        &mut self,
        token: &HTMLToken,
        is_special_tag: &impl Fn(tag_names, Namespace) -> bool,
    ) -> bool {
        let subject = token.as_tag().tag_name();

        if let Some(cnode) = self.current_node() {
            if cnode.element_ref().tag_name() == subject
                && !self
                    .list_of_active_formatting_elements
                    .contains_element(cnode)
            {
                self.stack_of_open_elements.pop();
                return false;
            }
        }

        let mut outer_loop_counter = 0;
        loop {
            if outer_loop_counter >= 8 {
                return false;
            }

            outer_loop_counter += 1;

            let maybe_formatting_element = self
                .list_of_active_formatting_elements
                .last_element_before_marker(subject);

            if maybe_formatting_element.is_none() {
                return true;
            }

            let (formatting_element_idx, formatting_element) =
                maybe_formatting_element.unwrap();

            if !self.stack_of_open_elements.contains(&formatting_element) {
                self.parse_error(token);
                self.list_of_active_formatting_elements
                    .remove(formatting_element_idx);
                return false;
            }

            if self.stack_of_open_elements.contains(&formatting_element)
                && !self.stack_of_open_elements.has_element_in_scope(
                    formatting_element.element_ref().tag_name(),
                    StackOfOpenElements::SCOPE_ELEMENTS,
                )
            {
                self.parse_error(token);
                return false;
            }

            if formatting_element
                .ne(self.current_node().expect("Le noeud actuel"))
            {
                self.parse_error(token);
            }

            let maybe_furthest_block = self
                .stack_of_open_elements
                .iter()
                .enumerate()
                .rfind(|(_, el)| {
                    if formatting_element.eq(el) {
                        return false;
                    }

                    let el = el.element_ref();
                    is_special_tag(
                        el.tag_name(),
                        el.namespace().expect(
                            "Devrait être un espace de nom valide",
                        ),
                    )
                })
                .map(|(i, e)| (i, e.to_owned()));

            if maybe_furthest_block.is_none() {
                while formatting_element
                    .ne(self.current_node().expect("Le noeud actuel"))
                {
                    self.stack_of_open_elements.pop();
                }

                self.stack_of_open_elements.pop();

                self.list_of_active_formatting_elements
                    .remove_element(&formatting_element);
                return false;
            }

            let (furthest_block_idx, furthest_block) =
                maybe_furthest_block.unwrap();

            let common_ancestor: Option<TreeNode<Node>> = {
                let mut found_node = None;
                for (index, element) in
                    self.stack_of_open_elements.iter().rev().enumerate()
                {
                    if formatting_element.eq(element) {
                        if index < self.stack_of_open_elements.len() - 1 {
                            found_node =
                                self.stack_of_open_elements.get(index - 1)
                        }
                        break;
                    }
                }
                found_node.cloned()
            };

            let mut bookmark = self
                .list_of_active_formatting_elements
                .iter()
                .rposition(|entry| match entry {
                    | Entry::Element(el) => formatting_element.eq(el),
                    | _ => false,
                })
                .unwrap();

            let mut node;
            let mut node_idx = furthest_block_idx;
            let mut last_node = furthest_block.to_owned();

            let mut inner_counter = 0;

            loop {
                inner_counter += 1;

                node = unsafe {
                    self.stack_of_open_elements.get_unchecked(node_idx)
                }
                .to_owned();
                node_idx -= 1;

                if formatting_element == node {
                    break;
                }

                if inner_counter > 3
                    && self
                        .list_of_active_formatting_elements
                        .contains_element(&node)
                {
                    self.list_of_active_formatting_elements
                        .remove_element(&node);
                    continue;
                }

                let node_formatting_index = {
                    if let Some(index) = self
                        .list_of_active_formatting_elements
                        .position_of(&node)
                    {
                        index
                    } else {
                        self.stack_of_open_elements
                            .remove_first_tag_matching(|n| node.eq(n));
                        continue;
                    }
                };

                let el = node.element_ref();
                let tag_token =
                    HTMLTagToken::start().with_name(el.local_name());
                let node_el = self
                    .create_element_for(
                        &tag_token,
                        el.namespace().expect(
                            "Devrait être un espace de nom valide",
                        ),
                        common_ancestor.as_ref(),
                    )
                    .expect("Devrait retourner un element valide");

                self.stack_of_open_elements[node_idx] = node_el.to_owned();
                self.list_of_active_formatting_elements
                    [node_formatting_index] =
                    Entry::Element(node_el.to_owned());

                node = node_el;

                if furthest_block.eq(&last_node) {
                    bookmark = node_formatting_index + 1;
                }

                node.append_child(last_node.to_owned());

                last_node = node;
            }

            let adjusted_insertion_location = self
                .find_appropriate_place_for_inserting_node(
                    common_ancestor.as_ref(),
                );

            if let Some(parent) = adjusted_insertion_location.parent {
                parent.insert_before(
                    last_node.to_owned(),
                    adjusted_insertion_location
                        .insert_before_sibling
                        .as_ref(),
                );
            }

            let el = node.element_ref();
            let tag_token =
                HTMLTagToken::start().with_name(el.local_name());
            let node_el = self
                .create_element_for(
                    &tag_token,
                    el.namespace()
                        .expect("Devrait être un espace de nom valide"),
                    Some(&furthest_block),
                )
                .expect("Devrait retourner un element valide");

            furthest_block.foreach_child(|child| {
                node_el.append_child(child.to_owned());
            });

            self.list_of_active_formatting_elements
                .remove_element(&formatting_element);
            self.list_of_active_formatting_elements[bookmark] =
                Entry::Element(node_el.to_owned());
            self.stack_of_open_elements
                .remove_first_tag_matching(|n| formatting_element.eq(n));
            self.stack_of_open_elements
                .insert(furthest_block_idx + 1, node_el);
        }
    }
}

impl HTMLTreeConstruction {
    fn handle_text_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token
            //
            // Insérer le caractère du jeton.
            //
            // Note: il ne peut jamais s'agir d'un caractère U+0000 NULL ;
            // le tokenizer les convertit en caractères
            // U+FFFD REPLACEMENT CHARACTER.
            | HTMLToken::Character(ch) => {
                self.insert_character(ch);
            }

            // An end-of-file token
            //
            // Erreur d'analyse.
            // Si le noeud actuel est un élément de type "script", alors
            // définir sa propriété `already_started` à true.
            // Retirer le noeud actuel de la pile d'éléments ouverts.
            // Passer le mode d'insertion au mode d'insertion original puis
            // retraiter le jeton.
            | HTMLToken::EOF => {
                self.parse_error(&token);

                if let Some(cnode) = self.current_node() {
                    let cnode_element = cnode.element_ref();
                    if tag_names::script == cnode_element.tag_name() {
                        cnode.script_ref().set_already_started(true);
                    }
                }

                self.stack_of_open_elements.pop();
                self.insertion_mode
                    .switch_to(self.original_insertion_mode);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // TODO: active spéculative html tree
            // An end tag whose tag name is "script"
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::script == name => {
                todo!()
            }

            // Any other end tag
            //
            // Retirer le nœud actuel de la pile des éléments ouverts.
            // Passer le mode d'insertion sur le mode d'insertion
            // d'origine.
            | HTMLToken::Tag(HTMLTagToken { is_end: true, .. }) => {
                self.stack_of_open_elements.pop();
                self.insertion_mode = self.original_insertion_mode;
            }

            // Rien n'est mentionné dans ce cas-ci dans la spécification.
            // Que faire ici?
            | _ => {}
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_table_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        /// Lorsque les étapes ci-dessous demandent à l'UA de vider la pile
        /// pour revenir à un contexte de tableau, cela signifie que l'UA
        /// doit, tant que le nœud actuel n'est pas un élément de tableau,
        /// de modèle ou html, extraire des éléments de la pile d'éléments
        /// ouverts.
        fn clear_stack_back_to_table_context(
            tree: &mut HTMLTreeConstruction,
        ) {
            while let Some(cnode) = tree.current_node() {
                if !cnode.element_ref().tag_name().is_one_of([
                    tag_names::table,
                    tag_names::template,
                    tag_names::html,
                ]) {
                    tree.stack_of_open_elements.pop();
                } else {
                    break;
                }
            }

            if let Some(cnode) = tree.current_node() {
                if cnode.element_ref().tag_name() == tag_names::html {
                    assert!(tree.parsing_fragment);
                }
            }
        }

        match token {
            // A character token, if the current node is table, tbody,
            // tfoot, thead, or tr element
            //
            // La table en attente de jetons de caractères doit être une
            // liste de jetons vide.
            // Le mode d'insertion d'origine est le mode d'insertion
            // actuel.
            // Passer le mode d'insertion à "in table text" puis retraiter
            // le jeton.
            | HTMLToken::Character(_)
                if self.current_node().is_some()
                    && !self
                        .current_node()
                        .unwrap()
                        .element_ref()
                        .tag_name()
                        .is_one_of([
                            tag_names::table,
                            tag_names::tbody,
                            tag_names::tfoot,
                            tag_names::thead,
                            tag_names::tr,
                        ]) =>
            {
                self.pending_table_character_tokens.clear();
                self.original_insertion_mode
                    .switch_to(self.insertion_mode);
                self.insertion_mode.switch_to(InsertionMode::InTableText);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A comment token
            //
            // Insérer un commentaire.
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "caption"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in caption".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::caption == name => {
                clear_stack_back_to_table_context(self);
                self.list_of_active_formatting_elements
                    .push(Entry::Marker);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InCaption);
            }

            // A start tag whose tag name is "colgroup"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in column group".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::colgroup == name => {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
            }

            // A start tag whose tag name is "col"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour un jeton de balise de début
            // "colgroup" sans attributs, puis passer le mode d'insertion à
            // "in column group".
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::col == name => {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(
                    &HTMLTagToken::start().with_name(tag_names::colgroup),
                );
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "tbody", "tfoot",
            // "thead"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in table body".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
            }

            // A start tag whose tag name is one of: "td", "th", "tr"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour un jeton de balise de début
            // "tbody" sans attributs, puis passer le mode d'insertion à
            // "in table body".
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::td,
                tag_names::th,
                tag_names::tr,
            ]) =>
            {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(
                    &HTMLTagToken::start().with_name(tag_names::tbody),
                );
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "table"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // table dans la portée de la table, nous devons ignorer le
            // jeton.
            // Sinon:
            // Retirer les éléments de cette pile jusqu'à ce qu'un élément
            // de table ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::table == name => {
                self.parse_error(&token);

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::table,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::table);
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // table dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton.
            // Sinon:
            // Retirer les éléments de cette pile jusqu'à ce qu'un élément
            // de table ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::table == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::table,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::table);
                self.reset_insertion_mode_appropriately();
            }

            // An end tag whose tag name is one of: "body", "caption",
            // "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
            // "thead", "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
                tag_names::tbody,
                tag_names::td,
                tag_names::tfoot,
                tag_names::th,
                tag_names::thead,
                tag_names::tr,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is one of: "style", "script",
            // "template"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::style,
                    tag_names::script,
                    tag_names::template,
                ])
                || is_end && tag_names::template == name =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // A start tag whose tag name is "input"
            //
            // Si le jeton ne possède pas d'attribut portant le nom "type",
            // ou s'il en possède un, mais que la valeur de cet attribut
            // n'est pas une correspondance ASCII insensible à la casse
            // pour la chaîne "hidden", alors : nous devons agir comme
            // décrit dans l'entrée "anything else" ci-dessous.
            // Sinon:
            // Erreur d'analyse.
            // Insérer un élément HTML pour le jeton.
            // Retirer cet élément d'entrée de la pile des éléments
            // ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    ref attributes,
                    is_end: false,
                    ..
                },
            ) if tag_names::input == name
                && attributes.iter().any(|attr| {
                    attr.name == "type"
                        && attr.value.eq_ignore_ascii_case("hidden")
                }) =>
            {
                self.parse_error(&token);
                let element = self.insert_html_element(tag_token);

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| element.contains(node),
                );

                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "form"
            //
            // Erreur d'analyse.
            // S'il existe un élément template sur la pile des éléments
            // ouverts, ou si le pointeur de l'élément de formulaire n'est
            // pas null, nous devons ignorer le jeton.
            // Sinon:
            // Insérer un élément HTML pour le jeton, et définir le
            // pointeur de l'élément form pour qu'il pointe sur l'élément
            // créé.
            // Retirer cet élément de formulaire de la pile des éléments
            // ouverts.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::form == name => {
                self.parse_error(&token);

                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                    || self.form_element_pointer.is_some()
                {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let element = self.insert_html_element(tag_token);
                self.form_element_pointer = element.clone();

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| element.contains(node),
                );
            }

            // An end-of-file token
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::EOF => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Activer le foster_parenting, traiter le
            // jeton en utilisant les règles du mode d'insertion "in body",
            // puis désactiver foster_parenting.
            | _ => {
                self.parse_error(&token);
                self.foster_parenting = true;
                self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
                self.foster_parenting = false;
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_table_text_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Character('\0') => {
                self.parse_error(&token);
                /* Ignore */
            }

            // Any other character token
            //
            // Ajouter le jeton de caractère à la liste des jetons de
            // caractère de la table en attente.
            | HTMLToken::Character(_) => {
                self.pending_table_character_tokens.push(token);
            }

            // Anything else
            //
            // Si l'un des jetons de la liste des jetons de caractères de
            // la table en attente est un jeton de caractère qui n'est pas
            // un espace blanc ASCII, il s'agit d'une erreur d'analyse :
            // retraiter les jetons de caractères de la liste des jetons de
            // caractères de la table en attente en appliquant les règles
            // données dans l'entrée "anything else" du mode d'insertion
            // "in table".
            // Sinon, insérer les caractères donnés par la liste des jetons
            // de caractères de la table en attente.
            // Passer le mode d'insertion au mode d'insertion original et
            // retraiter le jeton.
            | _ => {
                let pending_token_does_have_whitespace = self
                    .pending_table_character_tokens
                    .iter()
                    .any(|token| !token.is_ascii_whitespace());

                if pending_token_does_have_whitespace {
                    self.parse_error(&token);

                    for pending_token in
                        self.pending_table_character_tokens.clone()
                    {
                        // Jeton "Anything else" du mode d'insertion "in
                        // table"
                        self.foster_parenting = true;
                        self.process_using_the_rules_for(
                            InsertionMode::InBody,
                            pending_token,
                        );
                        self.foster_parenting = false;
                    }
                } else {
                    for pending_token in
                        self.pending_table_character_tokens.clone()
                    {
                        if let HTMLToken::Character(ch) = pending_token {
                            self.insert_character(ch);
                        }
                    }
                }

                self.insertion_mode
                    .switch_to(self.original_insertion_mode);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_caption_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // An end tag whose tag name is "caption"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // caption dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas du fragment)
            // Sinon:
            // Générer des balises de fin implicites.
            // Si le nœud actuel n'est pas un élément caption, il s'agit
            // d'une erreur d'analyse.
            // Retirer des éléments de cette pile jusqu'à ce qu'un élément
            // caption ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Passer le mode d'insertion à "in table".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::caption == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::caption,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::caption
                    {
                        self.parse_error(&token);
                    }
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::caption);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.insertion_mode.switch_to(InsertionMode::InTable);
            }

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr"
            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // caption dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas du fragment)
            // Sinon:
            // Générer des balises de fin implicites.
            // Si le nœud actuel n'est pas un élément caption, il s'agit
            // d'une erreur d'analyse.
            // Retirer des éléments de cette pile jusqu'à ce qu'un élément
            // caption ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Passer le mode d'insertion à "in table".
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::caption,
                    tag_names::col,
                    tag_names::colgroup,
                    tag_names::tbody,
                    tag_names::td,
                    tag_names::tfoot,
                    tag_names::th,
                    tag_names::thead,
                    tag_names::tr,
                ])
                || is_end && tag_names::table == name =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::caption,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::caption
                    {
                        self.parse_error(&token);
                    }
                }
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::caption);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "col",
            // "colgroup", "html", "tbody", "td", "tfoot", "th", "thead",
            // "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
                tag_names::tbody,
                tag_names::td,
                tag_names::tfoot,
                tag_names::th,
                tag_names::thead,
                tag_names::tr,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | _ => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_column_group_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION,
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF),
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // A comment token
            //
            // Insérer un commentaire
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is "col"
            //
            // Insérer un élément HTML pour le jeton. Extraire
            // immédiatement le nœud actuel de la pile
            // d'éléments ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::col == name => {
                self.insert_html_element(tag_token);
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // An end tag whose tag name is "colgroup"
            //
            // Si le nœud actuel n'est pas un élément colgroup, il s'agit
            // d'une erreur d'analyse ; ignorer le jeton.
            // Sinon, extraire le nœud actuel de la pile d'éléments
            // ouverts. Passer le mode d'insertion à "in table".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::colgroup == name => {
                if let Some(node) = self.current_node() {
                    if node.element_ref().tag_name() != tag_names::colgroup
                    {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTable);
            }

            // A end tag whose tag name is "col"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::col == name => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "template"
            // An end tag whose tag name is "template"
            //
            // Retraiter le jeton en utilisant les règles du mode
            // d'insertion "in head".
            | HTMLToken::Tag(HTMLTagToken { ref name, .. })
                if tag_names::template == name =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end-of-file token
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::EOF => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // Anything else
            //
            // Si le nœud actuel n'est pas un élément colgroup, il s'agit
            // d'une erreur d'analyse ; ignorer le jeton.
            // Sinon, extraire le nœud actuel de la pile d'éléments
            // ouverts. Passer le mode d'insertion à "in
            // table". Retraiter le jeton.
            | _ => {
                if let Some(node) = self.current_node() {
                    if node.element_ref().tag_name() != tag_names::colgroup
                    {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_table_body_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        /// Lorsque les étapes ci-dessous demandent à l'UA de vider la pile
        /// pour revenir à un contexte de corps de tableau, cela signifie
        /// que l'UA doit, tant que le nœud actuel n'est pas un élément
        /// tbody, tfoot, thead, template ou html, extraire des éléments de
        /// la pile des éléments ouverts.
        fn clear_stack_back_to_table_body_context(
            tree: &mut HTMLTreeConstruction,
        ) {
            while let Some(cnode) = tree.current_node() {
                if !cnode.element_ref().tag_name().is_one_of([
                    tag_names::tbody,
                    tag_names::tfoot,
                    tag_names::thead,
                    tag_names::template,
                    tag_names::html,
                ]) {
                    tree.stack_of_open_elements.pop();
                } else {
                    break;
                }
            }

            if let Some(cnode) = tree.current_node() {
                if cnode.element_ref().tag_name() == tag_names::html {
                    assert!(tree.parsing_fragment);
                }
            }
        }

        match token {
            // A start tag whose tag name is "tr"
            //
            // Effacer la pile pour revenir à un contexte "table body".
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in row".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::tr == name => {
                clear_stack_back_to_table_body_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InRow);
            }

            // A start tag whose tag name is one of: "th", "td"
            //
            // Erreur d'analyse.
            // Effacer la pile pour revenir à un contexte "table body".
            // Insérer un élément HTML pour un jeton de balise de début
            // "tr" sans attributs, puis passer le mode d'insertion à
            // "in row".
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([tag_names::th, tag_names::td]) => {
                clear_stack_back_to_table_body_context(self);
                self.insert_html_element(
                    &HTMLTagToken::start().with_name(tag_names::tr),
                );
                self.insertion_mode.switch_to(InsertionMode::InRow);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A end tag whose tag name is one of: "tbody", "tfoot",
            // "thead"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément dans
            // la portée de la table qui soit un élément HTML ayant le même
            // nom de balise que le jeton, il s'agit d'une erreur d'analyse
            // ; ignorer le jeton.
            // Sinon:
            // Effacer la pile pour revenir à un contexte "table body".
            // Retirer le nœud actuel de la pile d'éléments ouverts. Passer
            // le mode d'insertion à "in table".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    /* Ignore */
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                clear_stack_back_to_table_body_context(self);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTable);
            }

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "tbody", "tfoot", "thead"
            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // "tbody", "thead" ou "tfoot" dans la portée de la table, il
            // s'agit d'une erreur d'analyse ; ignorer le jeton.
            // Sinon:
            // Effacer la pile pour revenir à un contexte "table body".
            // Retirer le nœud actuel de la pile d'éléments ouverts. Passer
            // le mode d'insertion à "in table".
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::caption,
                    tag_names::col,
                    tag_names::colgroup,
                    tag_names::tbody,
                    tag_names::tfoot,
                    tag_names::thead,
                ])
                || is_end && tag_names::table == name =>
            {
                if !self.stack_of_open_elements.has_elements_in_scope(
                    [tag_names::tbody, tag_names::thead, tag_names::tfoot],
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                clear_stack_back_to_table_body_context(self);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "caption",
            // "col", "colgroup", "html", "td", "th", "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
                tag_names::td,
                tag_names::th,
                tag_names::tr,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // Anything else
            //
            // Retraiter le jeton en utilisant les règles du mode
            // d'insertion "in table".
            | _ => {
                return self.process_using_the_rules_for(
                    InsertionMode::InTable,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_row_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        /// Lorsque les étapes ci-dessus exigent que l'UA vide la pile pour
        /// revenir à un contexte de ligne de table, cela signifie que l'UA
        /// doit, tant que le nœud actuel n'est pas un élément tr, template
        /// ou html, extraire des éléments de la pile d'éléments ouverts.
        fn clear_stack_back_to_table_row_context(
            tree: &mut HTMLTreeConstruction,
        ) {
            while let Some(cnode) = tree.stack_of_open_elements.pop() {
                if !cnode.element_ref().tag_name().is_one_of([
                    tag_names::tr,
                    tag_names::template,
                    tag_names::html,
                ]) {
                    tree.stack_of_open_elements.pop();
                } else {
                    break;
                }
            }
        }
        match token {
            // A start tag whose tag name is one of: "th", "td"
            //
            // Effacer la pile pour revenir à un contexte "table row".
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in cell".
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([tag_names::th, tag_names::td]) => {
                clear_stack_back_to_table_row_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InCell);
                self.list_of_active_formatting_elements
                    .push(Entry::Marker);
            }

            // An end tag whose tag name is "tr"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément tr
            // dans la portée de la table, il s'agit d'une erreur d'analyse
            // ; ignorer le jeton.
            // Sinon:
            // Effacer la pile pour revenir à un contexte "table row".
            // Retirer le nœud actuel (qui sera un élément tr) de la pile
            // des éléments ouverts. Passer le mode d'insertion à
            // "in table body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::tr == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::tr,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                clear_stack_back_to_table_row_context(self);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
            }

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "tbody", "tfoot", "thead", "tr"
            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément tr
            // dans la portée de la table, il s'agit d'une erreur d'analyse
            // ; ignorer le jeton.
            // Sinon:
            // Effacer la pile pour revenir à un contexte "table row".
            // Retirer le nœud actuel (qui sera un élément tr) de la pile
            // des éléments ouverts. Passer le mode d'insertion à
            // "in table body".
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::caption,
                    tag_names::col,
                    tag_names::colgroup,
                    tag_names::tbody,
                    tag_names::tfoot,
                    tag_names::thead,
                    tag_names::tr,
                ])
                || is_end && tag_names::table == name =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::tr,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                clear_stack_back_to_table_row_context(self);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "tbody", "tfoot",
            // "thead"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément dans
            // la portée de la table qui soit un élément HTML ayant le même
            // nom de balise que le jeton, il s'agit d'une erreur d'analyse
            // ; ignorer le jeton.
            // Si la pile d'éléments ouverts ne comporte pas d'élément tr
            // dans la portée de la table, ignorer le jeton.
            // Sinon:
            // Effacer la pile pour revenir à un contexte "table row".
            // Retirer le nœud actuel (qui sera un élément tr) de la pile
            // des éléments ouverts. Passer le mode d'insertion à
            // "in table body".
            // Retraiter le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::tr,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                clear_stack_back_to_table_row_context(self);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "caption",
            // "col", "colgroup", "html", "td", "th"
            //
            // Erreur d'analyse: ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
                tag_names::td,
                tag_names::th,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in table".
            | _ => {
                return self.process_using_the_rules_for(
                    InsertionMode::InTable,
                    token,
                );
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_cell_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        /// Lorsque les étapes ci-dessous indiquent de fermer la cellule,
        /// elles signifient qu'il faut exécuter l'algorithme suivant :
        ///
        /// 1. Générer des balises de fin implicites.
        /// 2. Si le nœud actuel n'est pas un élément td ou un élément th,
        /// il s'agit d'une erreur d'analyse.
        /// 3. Retirer des éléments de la pile d'éléments ouverts jusqu'à
        /// ce qu'un élément td ou un élément th ait été retiré de la
        /// pile.
        /// 4. Effacer la liste des éléments de mise en forme actifs
        /// jusqu'au dernier marqueur.
        /// 5. Passer le mode d'insertion à "in row".
        fn close_cell(tree: &mut HTMLTreeConstruction, token: &HTMLToken) {
            tree.generate_implied_end_tags();

            if let Some(cnode) = tree.current_node() {
                if !cnode
                    .element_ref()
                    .tag_name()
                    .is_one_of([tag_names::td, tag_names::th])
                {
                    tree.parse_error(token);
                }
            }

            tree.stack_of_open_elements
                .pop_until_tags([tag_names::td, tag_names::th]);
            tree.list_of_active_formatting_elements
                .clear_up_to_the_last_marker();
            tree.insertion_mode.switch_to(InsertionMode::InRow);
        }

        match token {
            // An end tag whose tag name is one of: "td", "th"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément dans
            // la portée de la table qui soit un élément HTML ayant le même
            // nom de balise que celui du jeton, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton.
            // Sinon:
            // Générer des balises de fin implicites.
            // Maintenant, si le nœud actuel n'est pas un élément HTML avec
            // le même nom de balise que le jeton, il s'agit d'une erreur
            // d'analyse.
            // Extraire des éléments de la pile d'éléments ouverts jusqu'à
            // ce qu'un élément HTML ayant le même nom de balise que le
            // jeton ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Passer le mode d'insertion sur "in row".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([tag_names::td, tag_names::th]) => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name()
                        != tag_token.tag_name()
                    {
                        self.parse_error(&token);
                    }
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_token.tag_name());
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.insertion_mode.switch_to(InsertionMode::InRow);
            }

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément td
            // ou th dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas d'un fragment)
            // Sinon, nous devons fermer la cellule puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::tbody,
                tag_names::td,
                tag_names::tfoot,
                tag_names::th,
                tag_names::thead,
                tag_names::tr,
            ]) =>
            {
                if !self.stack_of_open_elements.has_elements_in_scope(
                    [tag_names::td, tag_names::th],
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                close_cell(self, &token);

                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "caption",
            // "col", "colgroup", "html"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // An end tag whose tag name is one of: "table", "tbody",
            // "tfoot", "thead", "tr"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément dans
            // la portée de la table qui soit un élément HTML ayant le même
            // nom de balise que celui du jeton, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton.
            // Sinon, nous devons fermer la cellule puis retraiter le
            // jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
            ]) =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                close_cell(self, &token);

                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | _ => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_select_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Character('\0') => {
                self.parse_error(&token);
                /* Ignore */
            }

            // Any other character token
            //
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) => {
                self.insert_character(ch);
            }

            // A comment token
            //
            // Insérer un commentaire.
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is "option"
            //
            // Si le noeud actuel est un élément d'option, il faut retirer
            // ce noeud de la pile des éléments ouverts.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::option == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is "optgroup"
            //
            // Si le noeud actuel est un élément option, il faut retirer
            // ce noeud de la pile des éléments ouverts.
            // Si le noeud actuel est un élément optgroup, il faut
            // retirer ce noeud de la pile des éléments ouverts.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::optgroup == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name()
                        == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                self.insert_html_element(tag_token);
            }

            // An end tag whose tag name is "optgroup"
            //
            // Tout d'abord, si le nœud actuel est un élément d'option et
            // que le nœud qui le précède immédiatement dans la pile des
            // éléments ouverts est un élément de groupe d'options, il faut
            // sortir le nœud actuel de la pile des éléments ouverts.
            // Si le noeud actuel est un élément d'optgroup, alors il faut
            // sortir ce noeud de la pile des éléments ouverts. Sinon, il
            // s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::optgroup == name => {
                if let (Some(pnode), Some(cnode)) =
                    (self.before_current_node(), self.current_node())
                {
                    let pelement = pnode.element_ref();
                    let celement = cnode.element_ref();

                    if celement.tag_name() == tag_names::option
                        && pelement.tag_name() == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name()
                        == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    } else {
                        self.parse_error(&token);
                        /* Ignore */
                    }
                }
            }

            // An end tag whose tag name is "option"
            //
            // Si le noeud actuel est un élément option, alors il faut
            // sortir ce noeud de la pile des éléments ouverts. Sinon, il
            // s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::option == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    } else {
                        self.parse_error(&token);
                        /* Ignore */
                    }
                }
            }

            // An end tag whose tag name is "select"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // select dans la portée select, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas d'un fragment)
            // Sinon:
            // Retirer des éléments de la pile d'éléments ouverts jusqu'à
            // ce qu'un élément select ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            //
            // Note: Il est juste traité comme une balise de fin.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::select == name => {
                if !self
                    .stack_of_open_elements
                    .has_element_in_scope_except(
                        tag_names::select,
                        StackOfOpenElements::select_scope_elements(),
                    )
                {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
            }

            // A start tag whose tag name is one of: "input", "keygen",
            // "textarea"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // select dans la portée select, nous devons ignorer le jeton.
            // (cas du fragment)
            // Sinon:
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément sélect ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::input,
                tag_names::keygen,
                tag_names::textarea,
            ]) =>
            {
                if !self
                    .stack_of_open_elements
                    .has_element_in_scope_except(
                        tag_names::select,
                        StackOfOpenElements::select_scope_elements(),
                    )
                {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "script", "template"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::script,
                    tag_names::template,
                ])
                || is_end && tag_names::template == name =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end-of-file token
            //
            // Traite rle jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::EOF => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Ignorer le jeton.
            | _ => {
                self.parse_error(&token);
                /* Ignore */
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_select_in_table_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A start tag whose tag name is one of: "caption", "table",
            // "tbody", "tfoot", "thead", "tr", "td", "th"
            //
            // Erreur d'analyse.
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément select ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::caption,
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
                tag_names::td,
                tag_names::th,
            ]) =>
            {
                self.parse_error(&token);
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                )
            }

            // An end tag whose tag name is one of: "caption", "table",
            // "tbody", "tfoot", "thead", "tr", "td", "th"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne contient pas d'élément dans
            // la portée de la table qui soit un élément HTML avec le même
            // nom de balise que celui du jeton, alors nous devons ignorer
            // le jeton.
            // Sinon:
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément sélect ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([
                tag_names::caption,
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
                tag_names::td,
                tag_names::th,
            ]) =>
            {
                self.parse_error(&token);

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                )
            }

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in select".
            | _ => self.process_using_the_rules_for(
                InsertionMode::InSelect,
                token,
            ),
        }
    }

    fn handle_in_template_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token
            // A comment token
            // A DOCTYPE token
            //
            // Traiter le jeton selon les règles du mode d'insertion
            // "in body".
            | HTMLToken::Character(_)
            | HTMLToken::Comment(_)
            | HTMLToken::DOCTYPE(_) => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is one of: "base", "basefont",
            // "bgsound", "link", "meta", "noframes", "script", "style",
            // "template", "title"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton selon les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::base,
                    tag_names::basefont,
                    tag_names::bgsound,
                    tag_names::link,
                    tag_names::meta,
                    tag_names::noframes,
                    tag_names::script,
                    tag_names::style,
                    tag_names::template,
                    tag_names::title,
                ])
                || is_end && tag_names::template == name =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // A start tag whose tag name is one of: "caption", "colgroup",
            // "tbody", "tfoot", "thead"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in table" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in table", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::caption,
                tag_names::colgroup,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTable);
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "col"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in column group" sur la pile des modes d'insertion
            // de template de sorte qu'il soit le nouveau mode
            // d'insertion de template actuel.
            // Passer le mode d'insertion à "in column group", puis
            // retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::col == name => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InColumnGroup);
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "tr"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in table body" sur la pile des modes d'insertion
            // de template de sorte qu'il soit le nouveau mode
            // d'insertion de template actuel.
            // Passer le mode d'insertion à "in table body", puis
            // retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::tr == name => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTableBody);
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "td", "th"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in row" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in row", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([tag_names::td, tag_names::th]) => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InRow);
                self.insertion_mode.switch_to(InsertionMode::InRow);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Any other start tag
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in body" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in body", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken { is_end: false, .. }) => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InBody);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken { is_end: true, .. }) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // An end-of-file token
            //
            // S'il n'y a pas d'élément template sur la pile des éléments
            // ouverts, alors nous devons arrêter l'analyse. (cas du
            // fragment)
            // Sinon, il s'agit d'une erreur d'analyse.
            // Retirer des éléments de la pile d'éléments ouverts jusqu'à
            // ce qu'un élément template ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Supprimer le mode d'insertion de template actuel de la pile
            // des modes d'insertion de template.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::EOF => {
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return HTMLTreeConstructionControlFlow::Break(
                        HTMLParserFlag::Stop,
                    );
                }

                self.parse_error(&token);
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::template);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_after_body_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant du premier
            // élément de la pile d'éléments ouverts (l'élément html).
            | HTMLToken::Comment(comment) => {
                let maybe_insertion_location =
                    self.stack_of_open_elements.first();
                if let Some(insertion_location) = maybe_insertion_location
                {
                    let comment =
                        CommentNode::new(&self.document, comment);
                    insertion_location.append_child(comment.to_owned());
                }
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end tag whose tag name is "html"
            //
            // Si l'analyseur a été créé dans le cadre de l'algorithme
            // d'analyse des fragments HTML, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton (cas du fragment).
            // Sinon, nous devons passer le mode d'insertion sur
            // "after after body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::html == name => {
                if self.parsing_fragment {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.insertion_mode
                    .switch_to(InsertionMode::AfterAfterBody);
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Passer le mode d'insertion à "in body" et
            // retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_in_frameset_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // A comment token
            //
            // Insérer un commentaire.
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is "frameset"
            //
            // Insérer un élément HTML pour le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::frameset == name => {
                self.insert_html_element(tag_token);
            }

            // An end tag whose tag name is "frameset"
            //
            // Si le nœud actuel est l'élément html racine, il s'agit d'une
            // erreur d'analyse ; ignorer le jeton (cas d'un fragment).
            // Sinon, extraire le nœud de la pile d'éléments ouverts.
            // Si l'analyseur syntaxique n'a pas été créé dans le cadre de
            // l'algorithme d'analyse syntaxique des fragments HTML (cas
            // des fragments) et que le nœud actuel n'est plus un élément
            // frameset, le mode d'insertion doit alors passer à "after
            // frameset".
            #[allow(deprecated)] // frameset
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::frameset == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::html {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                self.stack_of_open_elements.pop();

                if !self.parsing_fragment {
                    self.insertion_mode
                        .switch_to(InsertionMode::AfterFrameset);
                }
            }

            // A start tag whose tag name is "frame"
            //
            // Insérer un élément HTML pour le jeton. Extraire
            // immédiatement le nœud de la pile d'éléments ouverts.
            // Accuser réception du drapeau de fermeture automatique du
            // jeton, si défini.
            #[allow(deprecated)] // frame
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::frame == name => {
                self.insert_html_element(tag_token);
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "noframes"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)] // noframes
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::noframes == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end-of-file token
            //
            // Si le nœud actuel n'est pas l'élément html racine, il s'agit
            // d'une erreur d'analyse.
            // Note: Le nœud actuel ne peut être que l'élément html racine
            // dans le cas d'un fragment.
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::html {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Ignorer le jeton.
            | _ => {
                self.parse_error(&token);
                /* Ignore */
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_after_frameset_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C
            // FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // A comment token
            //
            // Insérer un commentaire.
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end tag whose tag name is "html"
            //
            // Passer le mode d'insertion à "after after frameset".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::html == name => {
                self.insertion_mode
                    .switch_to(InsertionMode::AfterAfterFrameset);
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Ignorer le jeton.
            | _ => {
                self.parse_error(&token);
                /* Ignore */
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_after_after_body_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // Document.
            | HTMLToken::Comment(comment) => {
                let comment = CommentNode::new(&self.document, comment);
                self.document.append_child(comment.to_owned());
            }

            // A DOCTYPE token,
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE,
            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::DOCTYPE(_) => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Passer le mode d'insertion à "in body" et
            // retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    fn handle_after_after_frameset_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // [Document].
            | HTMLToken::Comment(comment) => {
                let comment = CommentNode::new(&self.document, comment);
                self.document.append_child(comment.to_owned());
            }

            // A DOCTYPE token
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::DOCTYPE(_) => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // A start tag whose tag name is "noframes"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::noframes == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Ignorer le jeton.
            | _ => {
                self.parse_error(&token);
                /* Ignore */
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use dom::node::QuirksMode;

    use super::*;
    use crate::HTMLParser;

    macro_rules! load_fixture {
        ($filename:literal) => {{
            let document_node = DocumentNode::new();
            let html_file = include_str!($filename);
            HTMLParser::new(document_node, html_file.chars())
        }};
    }

    macro_rules! test_the_str {
        ($str:literal) => {{
            let document_node = DocumentNode::new();
            let html_file = $str;
            HTMLParser::new(document_node, html_file.chars())
        }};
    }

    #[test]
    #[should_panic]
    fn test_parse_document() {
        let mut parser = load_fixture!("../crashtests/test.html");
        parser.run();
        panic!(
            "[pour le test]: je veux paniquer même si tout fonctionne !"
        );
    }

    #[test]
    fn test_initial_insertion_mode() {
        // Comment token

        let mut parser = test_the_str!("<!-- Comment -->");
        let token = parser.tokenizer.next_token().unwrap();
        parser
            .tree_construction()
            .handle_initial_insertion_mode(token);
        let node = parser
            .tree_construction()
            .document
            .get_first_child()
            .to_owned()
            .unwrap();
        assert!(node.is_comment());
        assert!(!node.is_document());

        // Doctype

        let mut parser = test_the_str!("<!DOCTYPE html>");
        let token = parser.tokenizer.next_token().unwrap();
        let tree = parser.tree_construction();
        tree.handle_initial_insertion_mode(token);
        let doc = tree.document.document_ref();
        let doctype = doc.get_doctype().to_owned().unwrap();
        assert_eq!(doctype.name.to_string(), "html".to_string());
        assert_eq!(doctype.public_id.to_string(), "".to_string());
        assert_eq!(doctype.system_id.to_string(), "".to_string());

        // Anything else

        let mut parser = test_the_str!("a");
        let token = parser.tokenizer.next_token().unwrap();
        let tree = parser.tree_construction();
        tree.handle_initial_insertion_mode(token);
        let doc = tree.document.document_ref();
        assert_eq!(*doc.quirks_mode.read().unwrap(), QuirksMode::Yes);
        assert_eq!(tree.insertion_mode, InsertionMode::BeforeHTML);
    }

    #[test]
    fn test_before_html_insertion_mode() {
        // Comment

        let mut parser = test_the_str!("<!-- comment -->");
        let token = parser.tokenizer.next_token().unwrap();
        let tree = parser.tree_construction();
        tree.handle_before_html_insertion_mode(token);
        let doc = tree.document.get_first_child().to_owned().unwrap();
        assert!(doc.is_comment());

        // Tag

        let mut parser = test_the_str!("<html><head>");
        // <html>
        let token = parser.tokenizer.next_token().unwrap();
        let tree = parser.tree_construction();
        tree.handle_before_html_insertion_mode(token);
        let doc = tree.document.get_first_child().to_owned().unwrap();
        assert_eq!(tag_names::html, doc.element_ref().local_name());
        assert_eq!(tree.insertion_mode, InsertionMode::BeforeHead);

        // Anything else (<heap>)

        let token = parser.tokenizer.next_token().unwrap();
        let tree = parser.tree_construction();
        tree.handle_before_html_insertion_mode(token);
        let doc = tree.document.get_last_child().to_owned().unwrap();
        assert_eq!(tag_names::html, doc.element_ref().local_name());
        assert_ne!(tag_names::head, doc.element_ref().local_name());
    }
}
