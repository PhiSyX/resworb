/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

// --------- //
// Interface //
// --------- //

pub trait PeekableInterface<T, I>
where
    T: Iterator<Item = I>,
{
    /// Récupère le prochain élément de l'itération sans avancer dans
    /// l'itération.
    fn peek(&mut self) -> Option<&I>;

    /// Récupère les prochains éléments de l'itération jusqu'à une
    /// certaine position dans l'itération, sans avancer dans
    /// l'itération.
    ///
    /// Le type générique est obligatoire.
    ///
    /// NOTE(phisyx): ce type vaut None si la position demandée est
    /// supérieure à la longueur de l'itération, tant bien qu'il y ait
    /// des éléments à l'intérieur du flux. Exemple: si `lookahead_offset`
    /// vaut 5, et le flux a dans son flux `[0, 1, 2]` cela retournera
    /// `None`.
    fn peek_until<R: FromIterator<I>>(
        &mut self,
        lookahead_offset: usize,
    ) -> Option<R>;

    /// Récupère les prochains éléments de l'itération jusqu'à la fin de
    /// l'itération, sans avancer dans l'itération.
    ///
    /// Le type générique est obligatoire.
    fn peek_until_end<R: FromIterator<I>>(&mut self) -> R;

    /// Récupère les prochains éléments de l'itération entre deux positions
    /// de l'itération, sans avancer dans l'itération.
    ///
    /// Le type générique est obligatoire.
    fn peek_range(&mut self, range: Range<usize>) -> &[Option<I>];
}
