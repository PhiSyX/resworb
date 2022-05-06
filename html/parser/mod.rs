/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod codepoint;
mod error;
mod state;
mod token;
mod tokenizer;

use dom::{
    comment::Comment,
    doctype::DocumentType,
    document::{Document, QuirksMode},
};
use infra::primitive::codepoint::CodePoint;

use self::{
    state::{
        InsertionMode, ListOfActiveFormattingElements, StackOfOpenElements,
    },
    token::{HTMLTagToken, HTMLToken},
    tokenizer::HTMLTokenizer,
};

// --------- //
// Structure //
// --------- //

pub struct HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    tokenizer: HTMLTokenizer<C>,
    document: Document,
    insertion_mode: InsertionMode,
    stack_of_open_elements: StackOfOpenElements,
    stop_parsing: bool,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn new(document: Document, input: C) -> Self {
        let tokenizer = HTMLTokenizer::new(input);

        Self {
            tokenizer,
            document,
            insertion_mode: InsertionMode::default(),
            stack_of_open_elements: StackOfOpenElements::default(),
            stop_parsing: false,
        }
    }
}

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn run(&mut self) {
        loop {
            match self.tokenizer.next_token() {
                | None | Some(HTMLToken::EOF) => break,
                | Some(token) if self.stack_of_open_elements.is_empty() => {
                    self.process_using_the_rule_for(token);
                }
                | Some(token) => {
                    println!("Test {:?}", token);
                }
            }

            if self.stop_parsing {
                break;
            }
        }
    }

    fn process_using_the_rule_for(&mut self, token: HTMLToken) {
        match dbg!(&self.insertion_mode) {
            | InsertionMode::Initial => {
                self.handle_initial_insertion_mode(token)
            }
            | InsertionMode::BeforeHTML => todo!(),
            | InsertionMode::BeforeHead => todo!(),
            | InsertionMode::InHead => todo!(),
            | InsertionMode::InHeadNoscript => todo!(),
            | InsertionMode::AfterHead => todo!(),
            | InsertionMode::InBody => todo!(),
            | InsertionMode::Text => todo!(),
            | InsertionMode::InTable => todo!(),
            | InsertionMode::InTableText => todo!(),
            | InsertionMode::InCaption => todo!(),
            | InsertionMode::InColumnGroup => todo!(),
            | InsertionMode::InTableBody => todo!(),
            | InsertionMode::InRow => todo!(),
            | InsertionMode::InCell => todo!(),
            | InsertionMode::InSelect => todo!(),
            | InsertionMode::InSelectInTable => todo!(),
            | InsertionMode::InTemplate => todo!(),
            | InsertionMode::AfterBody => todo!(),
            | InsertionMode::InFrameset => todo!(),
            | InsertionMode::AfterFrameset => todo!(),
            | InsertionMode::AfterAfterBody => todo!(),
            | InsertionMode::AfterAfterFrameset => todo!(),
        }
    }

    fn switch_insertion_mode_to(
        &mut self,
        mode: InsertionMode,
    ) -> &mut Self {
        self.insertion_mode = mode;
        self
    }

    fn parse_error(&self, token: HTMLToken) {
        match token {
            | HTMLToken::Tag(HTMLTagToken {
                name, is_end_token, ..
            }) => {
                if is_end_token {
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
}

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    // TODO: si le document n'est pas un document iframe srcdoc
    fn handle_initial_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Ignorer le jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                /* ignore */
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // Document.
            | HTMLToken::Comment(comment) => {
                let comment = Comment::new(&self.document, comment);
                self.document.append_child(comment.node());
            }

            // A DOCTYPE token
            //
            // Si le nom du [HTMLToken::DOCTYPE] n'est pas "html", ou si
            // l'identificateur public du token n'est pas manquant, ou si
            // l'identificateur système du token n'est ni manquant ni
            // "about:legacy-compat", alors il y a une erreur d'analyse.
            //
            // Ajouter un noeud [DocumentType] au noeud [Document], avec
            // son nom défini comme le nom donné dans le
            // [HTMLToken::DOCTYPE], ou la chaîne de caractères vide si le
            // nom est manquant ; son ID public défini comme l'identifiant
            // public donné dans le [HTMLToken::DOCTYPE], ou la chaîne de
            // caractères vide si l'identifiant public est manquant ; et
            // son ID système défini comme l'identifiant système donné dans
            // le [HTMLToken::DOCTYPE], ou la chaîne de caractères vide si
            // l'identifiant système est manquant.
            //
            // Note: cela garantit également que le noeud [DocumentType]
            // est renvoyé comme valeur de l'attribut doctype de l'objet
            // [Document].
            //
            // Ensuite, si le document n'est pas un document iframe srcdoc,
            // et que l'analyseur syntaxique ne peut pas changer le drapeau
            // de mode est faux, et que le token DOCTYPE correspond à l'une
            // des conditions de la liste suivante, alors définir le
            // document en mode quirks :
            //   - Le drapeau force-quirks est activé.
            //   - Le nom du doctype n'est pas "html".
            //   - L'identifiant public est défini à l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_1]
            //   - L'identifiant système est défini à l'une des entrées du
            //     tableau [HTMLDoctypeToken::SYSTEM_ID_DEFINED_RULE_1]
            //   - L'identifiant public commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_STARTS_WITH_RULE_1]
            //   - L'identifiant système commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::SYSTEM_ID_STARTS_WITH_RULE_1]
            //
            // Sinon, si le document n'est pas un document iframe srcdoc,
            // que l'analyseur syntaxique ne peut pas modifier le drapeau
            // de mode est faux, et que le jeton DOCTYPE correspond à l'une
            // des conditions de la liste suivante, le document est alors
            // défini en mode limited-quirks :
            //   - L'identifiant public commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_2]
            //   - L'identifiant système n'est pas manquant et l'identifier
            //     public commence par l'une des entrées du tableau
            //     [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_2_1]
            //
            // Les chaînes de l'identifiant système et de l'identifiant
            // public doivent être comparées aux valeurs indiquées dans les
            // listes ci-dessus de manière insensible à la casse ASCII. Un
            // identifiant de système dont la valeur est la chaîne de
            // caractères vide n'est pas considéré comme manquant aux fins
            // des conditions ci-dessus.
            //
            // Ensuite, passer le mode d'insertion à "before html".
            | HTMLToken::DOCTYPE(doctype_data) => {
                let is_parse_error = doctype_data.is_html_name()
                    || !doctype_data.is_public_identifier_missing()
                    || !doctype_data.is_system_identifier_missing()
                    || !doctype_data.is_about_legacy_compat();

                if is_parse_error {
                    return;
                }

                let mut doctype = DocumentType::new(&self.document);
                doctype.set_name(doctype_data.name.as_ref());
                doctype.set_public_identifier(
                    doctype_data.public_identifier.as_ref(),
                );
                doctype.set_system_identifier(
                    doctype_data.system_identifier.as_ref(),
                );

                self.document.append_child(doctype.node());
                self.document.set_quirks_mode(doctype_data.quirks_mode());
                self.switch_insertion_mode_to(InsertionMode::BeforeHTML);
            }

            // Anything else
            //
            // Si le document n'est pas un document iframe srcdoc, il
            // s'agit d'une erreur d'analyse syntaxique ; si l'analyseur
            // syntaxique ne peut pas changer le drapeau de mode est faux,
            // mettez le document en mode quirks. Dans tous les
            // cas, passez le mode d'insertion à "before html", puis
            // retraitez le jeton.
            | _ => {
                self.parse_error(token);
                self.document.set_quirks_mode(QuirksMode::Yes);
                self.switch_insertion_mode_to(InsertionMode::BeforeHTML);
            }
        }
    }
}

