/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::{DocumentType, QuirksMode};

use crate::{
    state::InsertionMode,
    tokenization::HTMLToken,
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    // TODO(html): si le document n'est pas un document iframe srcdoc
    pub(crate) fn handle_initial_insertion_mode(
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
            // Ignorer le jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                /* Ignore */
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // [Document].
            | HTMLToken::Comment(comment) => {
                self.document.insert_comment(comment);
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
            // NOTE(html): cela garantit également que le noeud
            // [DocumentType] est renvoyé comme valeur de l'attribut
            // doctype de l'objet [Document].
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
            | HTMLToken::DOCTYPE { .. } => {
                let doctype_data = token.as_doctype();

                let is_parse_error = !doctype_data.is_html_name()
                    || !doctype_data.is_public_identifier_missing()
                    || !doctype_data.is_system_identifier_missing()
                        && !doctype_data.is_about_legacy_compat();

                if is_parse_error {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let mut doctype = DocumentType::new(doctype_data.name());

                doctype.set_public_id(doctype_data.public_identifier());
                doctype.set_system_id(doctype_data.system_identifier());

                self.document
                    .get_mut()
                    .set_doctype(doctype)
                    .set_quirks_mode(doctype_data.quirks_mode());
                self.insertion_mode.switch_to(InsertionMode::BeforeHTML);
            }

            // Anything else
            //
            // Si le document n'est pas un document iframe srcdoc, il
            // s'agit d'une erreur d'analyse syntaxique ; si l'analyseur
            // syntaxique ne peut pas changer le drapeau de mode est faux,
            // nous devons mettre le document en mode quirks. Et dans tous
            // les cas, passer le mode d'insertion à "before
            // html", puis retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.document.get_mut().set_quirks_mode(QuirksMode::Yes);
                self.insertion_mode.switch_to(InsertionMode::BeforeHTML);
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}
