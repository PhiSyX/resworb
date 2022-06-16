/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::algorithms::Parameter;

// --------- //
// Interface //
// --------- //

pub trait StreamIterator {
    type Item;

    /// Alias de [Iterator::nth] : sauf que l'on fait n - 1 si n est
    /// supérieur ou égal à 1. Ce qui veut dire que : (n = n - 1)
    ///   - 1 = 0
    ///   - 2 = 1
    ///   - 3 = 2
    /// etc...
    fn advance(&mut self, n: usize) -> Option<Self::Item> {
        unimplemented!("Avancer de {n} dans le flux.");
    }

    /// Avance dans le flux autant que possible, tant que le prédicat est
    /// vrai.
    ///
    /// Exemple:
    ///
    /// Le flux d'entrée de départ vaut `[' ', ' ', ' ', 'a', ' ', 'b']`;
    /// On veut avancer dans le flux, tant que le caractère suivant est un
    /// espace.
    ///
    /// Code:
    ///     advance_as_long_as_possible(|next_ch| next_ch.is_whitespace());
    ///
    /// Après cette opération, le flux d'entrée vaut `['a', ' ', 'b']`.
    #[allow(unused_variables)]
    fn advance_as_long_as_possible<
        'a,
        Predicate: Fn(&Self::Item) -> bool,
        Limit: Parameter<'a, usize>,
    >(
        &mut self,
        predicate: Predicate,
        with_limit: Limit,
    ) -> Vec<Self::Item> {
        unimplemented!(
            "Avance dans le flux autant que possible, tant que le prédicat \
            est vrai."
        );
    }
}

pub trait StreamInputIterator {
    type Input: StreamInput;

    /// Consomme la première entrée d'un flux.
    fn consume_next_input(&mut self) -> Option<Self::Input>;

    /// La dernière entrée d'un flux à avoir été consommée.
    fn current_input(&self) -> Option<&Self::Input>;

    /// La dernière entrée d'un flux à avoir été consommée (mutable).
    fn current_input_mut(&self) -> Option<&mut Self::Input> {
        unimplemented!(
            "La dernière entrée d'un flux à avoir été consommée (mutable)."
        );
    }

    /// Modifier l'entrée actuelle à partir d'une fonction
    /// de retour.
    #[allow(unused_variables)]
    fn current_input_mut_callback<F: FnOnce(&mut Self::Input)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        unimplemented!(
            "Possibilité de modifier l'entrée actuelle à partir d'une fonction \
            de retour."
        );
    }

    /// La première entrée d'un flux qui n'a pas encore été consommée.
    fn next_input(&mut self) -> Option<Self::Input>;

    /// Les N premières entrées d'un flux qui n'ont pas encore été
    /// consommées sous forme de tableau.
    fn next_n_input(&mut self, n: usize) -> Vec<Self::Input> {
        unimplemented!(
            "Les {n} premières entrées d'un flux qui n'ont pas encore été \
             consommées sous forme de tableau."
        );
    }

    /// Pousse (l'entrée actuelle](Self::current_input) à l'avant d'un
    /// flux, de sorte à ce que la prochaine fois qu'il sera demandé de
    /// consommer l'entrée suivante, il reprendra plutôt l'entrée actuelle.
    fn reconsume_current_input(&mut self);
}

pub trait StreamInput: PartialEq + Eq + Clone + std::fmt::Debug {
    /// Un EOF conceptuel représentant la fin du flux.
    /// Lorsque la liste est vide, la prochaine entrée est toujours un
    /// EOF.
    fn eof() -> Self;

    /// Est-ce que l'entrée est une fin de flux ?
    fn is_eof(&self) -> bool {
        *self == Self::eof()
    }
}

pub trait StreamTokenIterator {
    type Token: StreamToken;

    /// Consomme le premier jeton d'un flux.
    fn consume_next_token(&mut self) -> Option<Self::Token>;

    /// Le dernier jeton d'un flux à avoir été consommée.
    fn current_token(&self) -> Option<&Self::Token>;

    /// Le dernier dernier d'un flux à avoir été consommée (mutable).
    fn current_token_mut(&mut self) -> Option<&mut Self::Token> {
        unimplemented!(
            "Le dernier jeton d'un flux à avoir été consommée (mutable)."
        );
    }

    /// Modifier le jeton actuel à partir d'une fonction de retour.
    #[allow(unused_variables)]
    fn current_token_mut_callback<F: FnOnce(&mut Self::Token)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        unimplemented!(
            "Possibilité de modifier le jeton actuel à partir d'une fonction \
            de retour."
        );
    }

    /// Le premier jeton d'un flux qui n'a pas encore été consommée.
    fn next_token(&mut self) -> Option<Self::Token>;

    /// Les N premiers jetons d'un flux qui n'ont pas encore été
    /// consommées sous forme de tableau.
    fn next_n_token(&mut self, n: usize) -> Vec<Self::Token> {
        unimplemented!(
            "Les {n} premiers jetons d'un flux qui n'ont pas encore été \
             consommées sous forme de tableau."
        );
    }

    /// Pousse (l'entrée actuelle](Self::current_input) à l'avant d'un
    /// flux, de sorte à ce que la prochaine fois qu'il sera demandé de
    /// consommer l'entrée suivante, il reprendra plutôt l'entrée actuelle.
    fn reconsume_current_token(&mut self);
}

pub trait StreamToken: PartialEq + Eq + Clone + std::fmt::Debug {
    /// Un jeton conceptuel représentant la fin de la liste des jetons.
    /// Lorsque la liste de jetons est vide, le prochain jeton d'entrée est
    /// toujours un <EOF-token>.
    fn eof() -> Self;

    /// Est-ce que l'entrée est une fin de flux ?
    fn is_eof(&self) -> bool {
        *self == Self::eof()
    }
}
