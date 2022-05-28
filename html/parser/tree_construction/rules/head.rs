/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::{interface::IsOneOfTagsInterface, tag_names};
use infra::namespace::Namespace;

use crate::{
    state::{FramesetOkFlag, InsertionMode, ScriptingFlag},
    tokenization::{HTMLTagToken, HTMLToken, HTMLTokenizerState},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_before_head_insertion_mode(
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
                /* ignore */
            }

            // A comment token
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => self.parse_error(&token),

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

            // A start tag whose tag name is "head"
            //
            // Insérer un élément HTML pour le jeton.
            // Placer le pointeur de l'élément head sur le nouvel élément
            // head fraîchement créé.
            // Passer le mode d'insertion à "in head".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::head == name => {
                let head_element = self.insert_html_element(tag_token);
                self.head_element_pointer = head_element;
                self.insertion_mode.switch_to(InsertionMode::InHead);
            }

            // Une balise de fin dont le nom de balise est l'un des
            // éléments suivants: "head", "body", "html", "br".
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // Toute autre nom de balise de fin:
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if !name.is_one_of([
                tag_names::head,
                tag_names::body,
                tag_names::html,
                tag_names::br,
            ]) =>
            {
                self.parse_error(&token);
            }

            // Anything else
            //
            // Insérer un élément HTML pour un jeton de balise de début
            // "head" sans attributs.
            // Placer le pointeur de l'élément "head" sur l'élément "head"
            // fraîchement créé.
            // Passer le mode d'insertion à "in head".
            // Retraiter le jeton en cours.
            | _ => {
                let head_element =
                    HTMLTagToken::start().with_name(tag_names::head);
                self.head_element_pointer =
                    self.insert_html_element(&head_element);
                self.insertion_mode.switch_to(InsertionMode::InHead);
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

    pub(crate) fn handle_in_head_insertion_mode(
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
            // Insérer un caractère.
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
                /* ignore */
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

            // A start tag whose tag name is one of:
            //   "base", "basefont", "bgsound", "link"
            //
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le noeud actuel de la pile des éléments ouverts.
            // Accuser la réception du drapeau d'auto-fermeture du jeton,
            // s'il est activé.
            #[allow(deprecated)]
            | HTMLToken::Tag(mut tag_token)
                if !tag_token.is_end
                    && tag_token.name.is_one_of([
                        tag_names::base,
                        tag_names::basefont,
                        tag_names::bgsound,
                        tag_names::link,
                    ]) =>
            {
                self.insert_html_element(&tag_token);
                self.stack_of_open_elements.pop();
                tag_token.set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "meta"
            //
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le noeud actuel de la pile des éléments ouverts.
            // Accuser réception du drapeau d'auto-fermeture du jeton, s'il
            // est activé.
            //
            // TODO:
            // Si l'analyseur HTML spéculatif actif est nul, alors :
            //
            //   1. Si l'élément possède un attribut charset, et que
            // l'obtention d'un encodage à partir de sa valeur donne un
            // encodage, et que la confiance est actuellement provisoire,
            // alors nous devons changer l'encodage pour l'encodage
            // résultant.
            //
            //   2. Sinon, si l'élément possède un attribut http-equiv dont
            // la valeur est une correspondance ASCII insensible à la casse
            // pour la chaîne "Content-Type", et que l'élément possède un
            // attribut content, et que l'application de l'algorithme
            // d'extraction d'un encodage de caractères d'un méta-élément à
            // la valeur de cet attribut renvoie un encodage, et que la
            // confiance est actuellement provisoire, alors nous devons
            // changer l'encodage pour l'encodage extrait.
            //
            // Note: L'analyseur HTML spéculatif n'applique pas de manière
            // spéculative les déclarations de codage des caractères afin
            // de réduire la complexité de l'implémentation.
            | HTMLToken::Tag(mut tag_token)
                if !tag_token.is_end
                    && tag_names::meta == tag_token.name =>
            {
                self.insert_html_element(&tag_token);
                self.stack_of_open_elements.pop();
                tag_token.set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "title"
            //
            // Suivre l'algorithme générique d'analyse syntaxique de
            // l'élément RCDATA.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::title == name => {
                self.parse_generic_element(
                    tag_token,
                    HTMLTokenizerState::RCDATA,
                );
            }

            // A start tag whose tag name is "noscript", if the scripting
            // flag is enabled.
            // A start tag whose tag name is one of: "noframes", "style".
            //
            // Suivre l'algorithme générique d'analyse syntaxique des
            // éléments de texte brut (RAWTEXT).
            #[allow(deprecated)]
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if (tag_names::noscript == name
                && self.scripting_flag == ScriptingFlag::Enabled)
                || name.is_one_of([
                    tag_names::noframes,
                    tag_names::style,
                ]) =>
            {
                self.parse_generic_element(
                    tag_token,
                    HTMLTokenizerState::RAWTEXT,
                );
            }

            // A start tag whose tag name is "noscript", if the scripting
            // flag is disabled
            //
            // Insérer un élément HTML pour le jeton.
            // Passer le mode d'insertion à "in head noscript".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::noscript == name
                && self.scripting_flag == ScriptingFlag::Disabled =>
            {
                self.insert_html_element(tag_token);
                self.insertion_mode
                    .switch_to(InsertionMode::InHeadNoscript);
            }

            // A start tag whose tag name is "script"
            //
            //   1. Laisser l'emplacement d'insertion ajusté l'endroit
            // approprié pour insérer un nœud.
            //   2. Créer un élément pour le jeton dans l'espace de noms
            // HTML, le parent prévu étant l'élément dans lequel se trouve
            // l'emplacement d'insertion ajusté.
            //  3. Définir le document de l'analyseur de l'élément comme
            // étant le Document, et désactive l'indicateur "non-bloquant"
            // de l'élément.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::script == name => {
                let mut adjusted_insertion_location =
                    self.find_appropriate_place_for_inserting_node(None);

                let element = self
                    .create_element_for(
                        tag_token,
                        Namespace::HTML,
                        adjusted_insertion_location.parent.as_ref(),
                    )
                    .expect("Un élément HTMLScriptElement");

                let script_element = element
                    .script_ref()
                    .set_parser_document(&self.document)
                    .set_non_blocking(false);

                if self.parsing_fragment {
                    script_element.set_already_started(true);
                }

                // todo(document.write/ln): Si l'analyseur syntaxique a été
                // invoqué par l'intermédiaire des méthodes
                // document.write() ou document.writeln(), il est possible
                // de marquer l'élément script comme "déjà lancé". (Par
                // exemple, l'agent utilisateur peut utiliser cette clause
                // pour empêcher l'exécution de scripts d'origine croisée
                // insérés via document.write() dans des conditions de
                // réseau lent, ou lorsque le chargement de la page a déjà
                // pris beaucoup de temps).

                if let Some(ref mut parent) =
                    adjusted_insertion_location.parent
                {
                    parent.insert_before(
                        element.to_owned(),
                        adjusted_insertion_location
                            .insert_before_sibling
                            .as_ref(),
                    );
                }

                self.stack_of_open_elements.put(element);
                let token_state = HTMLTokenizerState::ScriptData;
                self.original_insertion_mode
                    .switch_to(self.insertion_mode);
                self.insertion_mode.switch_to(InsertionMode::Text);

                return HTMLTreeConstructionControlFlow::Continue(
                    HTMLParserState::SwitchTo(token_state.to_string()),
                );
            }

            // An end tag whose tag name is "head"
            //
            // Retirer le nœud actuel (qui sera l'élément de tête) de la
            // pile des éléments ouverts.
            // Passer le mode d'insertion sur "après la tête".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::head == name => {
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::AfterHead);
            }

            // A start tag whose tag name is "template"
            //
            // Insérer un élément HTML pour le jeton.
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            // Définir le drapeau frameset-ok à "not ok".
            // Passer le mode d'insertion sur "in template".
            // Pousser "in template" sur la pile des modes d'insertion
            // de template afin qu'il soit le nouveau mode d'insertion de
            // template actuel.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::template == name => {
                self.insert_html_element(tag_token);
                self.list_of_active_formatting_elements
                    .insert_marker_at_end();
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTemplate);
            }

            // An end tag whose tag name is "template"
            //
            // S'il n'y a pas d'élément template sur la pile des éléments
            // ouverts, il s'agit d'une erreur d'analyse ; ignorer le
            // jeton.
            //
            // Sinon:
            //   1. générer minutieusement toutes les balises de fin
            // implicites.
            //   2. Si le nœud actuel n'est pas un élément template, il
            // s'agit d'une erreur d'analyse.
            //   3. Extraire des éléments de la pile d'éléments ouverts
            // jusqu'à ce qu'un élément template ait été extrait de la
            // pile.
            //   4. Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            //   5. Supprimer le mode d'insertion template actuel de la
            // pile des modes d'insertion template.
            //   6. Réinitialiser le mode d'insertion de manière
            // appropriée.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::template == name => {
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_all_implied_end_tags_thoroughly();

                let element_name = self
                    .current_node()
                    .expect("Le noeud actuel")
                    .element_ref()
                    .local_name();

                if tag_names::template != element_name {
                    self.parse_error(&token);
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::template);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.stack_of_template_insertion_modes.pop();

                self.reset_insertion_mode_appropriately();
            }

            // A start tag whose tag name is "head"
            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if is_end || tag_names::head == name => {
                self.parse_error(&token)
            }

            // Anything else
            //
            // Retirer le nœud actuel (qui sera l'élément de tête) de la
            // pile des éléments ouverts.
            | _ => {
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::AfterHead);
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

    pub(crate) fn handle_in_head_noscript_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
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

            // An end tag whose tag name is "noscript"
            //
            // Extraire le noeud actuel (qui sera un élément noscript) de
            // la pile des éléments ouverts ; le nouveau noeud actuel sera
            // un élément head.
            // Passer le mode d'insertion à "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::noscript == name => {
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InHead);
            }

            // U+0009 CHARACTER
            // TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF),
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            // A comment token
            // A start tag whose tag name is one of: "basefont", "bgsound",
            // "link", "meta", "noframes", "style"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }
            | HTMLToken::Comment(_) => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::basefont,
                tag_names::bgsound,
                tag_names::link,
                tag_names::meta,
                tag_names::noframes,
                tag_names::style,
            ]) =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end tag whose tag name is "br"
            //
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // A start tag whose tag name is one of: "head", "noscript"
            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name
                    .is_one_of([tag_names::head, tag_names::noscript])
                || is_end && tag_names::br != name =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // Anything else
            //
            // Erreur d'analyse.
            // Extraire le noeud actuel (qui sera un élément noscript) de
            // la pile des éléments ouverts ; le nouveau noeud actuel sera
            // un élément head.
            // Passer le mode d'insertion sur "in head".
            // Retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::InHead);
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

    pub(crate) fn handle_after_head_insertion_mode(
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

            // A start tag whose tag name is "body"
            //
            // Insérer un élément HTML pour le jeton.
            // Définir le drapeau frameset-ok à "not ok".
            // Passer le mode d'insertion à "in body".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::body == name => {
                self.insert_html_element(tag_token);
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.insertion_mode.switch_to(InsertionMode::InBody);
            }

            // A start tag whose tag name is "frameset"
            //
            // Insérer un élément HTML pour le jeton.
            // Passer le mode d'insertion à "in frameset".
            #[allow(deprecated)]
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::frameset == name => {
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InFrameset);
            }

            // A start tag whose tag name is one of:
            // "base", "basefont", "bgsound", "link", "meta", "noframes",
            // "script", "style", "template", "title"
            //
            // Erreur d'analyse.
            // Pousser le nœud pointé par le pointeur de l'élément "head"
            // sur la pile des éléments ouverts.
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            // Retirer le noeud pointé par le pointeur de l'élément "head"
            // de la pile des éléments ouverts. (Il se peut que ce ne soit
            // pas le nœud actuel à ce stade).
            //
            // Note: le pointeur de l'élément de tête ne peut pas être nul
            // à ce stade.
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
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
            ]) =>
            {
                self.parse_error(&token);

                if let Some(head) = self.head_element_pointer.as_ref() {
                    self.stack_of_open_elements.put(head.to_owned());
                }

                self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| {
                        if let Some(head) =
                            self.head_element_pointer.as_ref()
                        {
                            return node == head;
                        }
                        false
                    },
                );

                assert!(matches!(self.head_element_pointer, Some(_)));
            }

            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::template == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "html", "br"
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // A start tag whose tag name is "head"
            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if (!is_end && tag_names::head == name)
                || (is_end
                    && !name.is_one_of([
                        tag_names::body,
                        tag_names::html,
                        tag_names::br,
                    ])) =>
            {
                self.parse_error(&token);
                /* ignore */
            }

            // Anything else
            //
            // Insérer un élément HTML pour un jeton de balise de début
            // "body" sans attributs.
            // Passer le mode d'insertion sur "in body".
            // Retraiter le jeton actuel.
            | _ => {
                let body_element =
                    HTMLTagToken::start().with_name(tag_names::body);
                self.insert_html_element(&body_element);
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
}
