/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{at_rule::CSSAtRule, qualified_rule::CSSQualifiedRule};

/// la production `<rule-list>` représente une liste de règles, et ne peut
/// être utilisée dans les grammaires que comme seule valeur dans un bloc.
/// Elle indique que le contenu du bloc doit être analysé à l'aide de
/// l'algorithme consume a list of rules.
// NOTE(phisyx): peut-être améliorer ce type.
pub type CSSRuleList = Vec<CSSRule>;

/// la production `<stylesheet>` représente une liste de règles. Elle est
/// identique à `<rule-list>`, sauf que les blocs qui l'utilisent acceptent
/// par défaut toutes les règles qui ne sont pas autrement limitées à un
/// contexte particulier.
// NOTE(phisyx): peut-être améliorer ce type.
pub type CSSStyleSheet = CSSRuleList;

// ----------- //
// Énumération //
// ----------- //

/// Voir le tableau <https://www.w3.org/TR/css-syntax-3/#declaration-rule-list>
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum CSSRule {
    QualifiedRule(CSSQualifiedRule),
    AtRule(CSSAtRule),
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSQualifiedRule> for CSSRule {
    fn from(qualified_rule: CSSQualifiedRule) -> Self {
        Self::QualifiedRule(qualified_rule)
    }
}

impl From<CSSAtRule> for CSSRule {
    fn from(at_rule: CSSAtRule) -> Self {
        Self::AtRule(at_rule)
    }
}
