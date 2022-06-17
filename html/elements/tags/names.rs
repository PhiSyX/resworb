/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};

use crate::interface::IsOneOfTagsInterface;

// ------ //
// Macros //
// ------ //

macro_rules! enumerate_html_tag_names {
    ($(
        $(#[$attr:meta])*
        $name:ident
    )*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        #[derive(Copy, Clone)]
        #[derive(PartialEq, Eq)]
        pub enum tag_names {
        $(
            #[allow(non_upper_case_globals)]
            #[doc = "Nom de la balise :"]
            #[doc = stringify!($name)]
            $(#[$attr])*
            $name
        ),*
        }

        impl str::FromStr for tag_names {
            type Err = &'static str;

            #[allow(deprecated)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $(| stringify!($name) => Self::$name),*,
                    | _ => return Err("Élément inconnu")
                })
            }
        }

        impl fmt::Display for tag_names {
            #[allow(deprecated)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),*
                })
            }
        }
    };
}

// -------------- //
// Implémentation //
// -------------- //

impl tag_names {
    /*
    NameStartChar ::= ":" | [A-Z]     | "_" | [a-z]     | [#xC0-#xD6]
                    | [#xD8-#xF6]     | [#xF8-#x2FF]    | [#x370-#x37D]
                    | [#x37F-#x1FFF]  | [#x200C-#x200D] | [#x2070-#x218F]
                    | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF]
                    | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
    */
    fn name_start_char(ch: char) -> bool {
        ch.is_ascii_alphabetic()
            || matches!(ch, | ':' | '_'
             | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}'
             | '\u{00F8}'..='\u{02FF}' | '\u{0370}'..='\u{037D}'
             | '\u{037F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}'
             | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}'
             | '\u{3001}'..='\u{D7FF}' | '\u{F901}'..='\u{FDCF}'
             | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}'
            )
    }

    /*
    NameChar :: = NameStartChar   | "-" | "." | [0-9] | #xB7
                | [#x0300-#x036F] | [#x203F-#x2040]
    */
    fn name_char(ch: char) -> bool {
        Self::name_start_char(ch)
            || ch.is_ascii_alphanumeric()
            || matches!(ch, '-' | '.'
             | '\u{00B7}'
             | '\u{0300}'..='\u{036F}'
             | '\u{203F}'..='\u{2040}'
            )
    }

    pub fn is_valid_name(name: impl AsRef<str>) -> bool {
        let name = name.as_ref();

        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();

        let next_ch = chars.next().unwrap();
        if !Self::name_start_char(next_ch) {
            return false;
        }

        chars.any(Self::name_char)
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl IsOneOfTagsInterface for tag_names {
    fn is_one_of(self, arr: impl IntoIterator<Item = Self>) -> bool {
        arr.into_iter().any(|tag_name| tag_name == self)
    }
}

impl<S> IsOneOfTagsInterface for S
where
    S: AsRef<str>,
    S: Copy,
{
    fn is_one_of(self, arr: impl IntoIterator<Item = tag_names>) -> bool {
        arr.into_iter().any(|tag_name| tag_name == self)
    }
}

impl<S> PartialEq<S> for tag_names
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        self.to_string().eq(other.as_ref())
    }
}

// ----------------------- //
// Application de la macro //
// ----------------------- //

enumerate_html_tag_names! {
    /// L'élément HTML <html> représente la racine d'un document HTML ou XHTML.
    /// Tout autre élément du document doit être un descendant de cet élément.
    html

    /// L'élément <base> définit l'URL de base à utiliser pour recomposer
    /// toutes les URL relatives contenues dans un document. Il ne peut y
    /// avoir qu'un seul élément <base> au sein d'un document.
    base

    /// L'élément HTML <head> fournit des informations générales (métadonnées)
    /// sur le document, incluant son titre et des liens ou des définitions
    /// vers des scripts et feuilles de style.
    head

    /// L'élément HTML <link> définit la relation entre le document courant et
    /// une ressource externe. Cet élément peut être utilisé pour définir un
    /// lien vers une feuille de style, vers les icônes utilisées en barre de
    /// titre ou comme icône d'application sur les appareils mobiles.
    link

    /// L'élément HTML <meta> représente toute information de métadonnées qui
    /// ne peut pas être représentée par un des éléments (base, link, script,
    /// style ou title)
    meta

    /// L'élément HTML <style> contient des informations de mise en forme pour
    /// un document ou une partie d'un document. Par défaut, les instructions
    /// de mise en forme écrites dans cet élément sont écrites en CSS.
    style

    /// L'élément <title> définit le titre du document (qui est affiché dans
    /// la barre de titre du navigateur ou dans l'onglet de la page).
    /// Cet élément ne peut contenir que du texte, les balises qu'il
    /// contiendrait seraient ignorées.
    title

    /// L'élément HTML <body> représente le contenu principal du document HTML.
    /// Il ne peut y avoir qu'un élément <body> par document.
    body


    /// L'élément HTML <address> indique des informations de contact pour une
    /// personne, un groupe de personnes ou une organisation.
    address

    /// L'élément <article> représente une composition autonome dans un
    /// document, une page, une application ou un site, destinée à être
    /// distribuée ou réutilisée de manière indépendante (par exemple, dans
    /// le cadre d'une syndication). Exemples : un message de forum, un article
    /// de magazine ou de journal, ou un article de blog, une fiche produit,
    /// un commentaire soumis par un utilisateur, un widget ou gadget
    /// interactif, ou tout autre élément de contenu indépendant.
    article

    /// L'élément <aside> (en français, « aparté ») représente une partie d'un
    /// document dont le contenu n'a qu'un rapport indirect avec le contenu
    /// principal du document. Les apartés sont fréquemment présents sous la
    /// forme d'encadrés ou de boîtes de légende.
    aside

    /// L'élément HTML <footer> représente le pied de page de la section ou de
    /// la racine de sectionnement la plus proche. Un élément <footer> contient
    /// habituellement des informations sur l'autrice ou l'auteur de la section,
    /// les données relatives au droit d'auteur (copyright) ou les liens vers
    /// d'autres documents en relation.
    footer

    /// L'élément HTML <header> représente du contenu introductif, généralement
    /// un groupe de contenu introductif ou de contenu aidant à la navigation.
    /// Il peut contenir des éléments de titre, mais aussi d'autres éléments
    /// tels qu'un logo, un formulaire de recherche, le nom d'auteur, etc.
    header

    /// Les éléments <h1> à <h6> représentent six niveaux de titres dans un
    /// document, <h1> est le plus important et <h6> est le moins important.
    /// Un élément de titre décrit brièvement le sujet de la section qu'il
    /// introduit.
    h1 h2 h3 h4 h5 h6

    /// L’élément HTML <main> représente le contenu majoritaire du body du
    /// document. Le contenu principal de la zone est constitué de contenu
    /// directement en relation, ou qui étend le sujet principal du document
    /// ou de la fonctionnalité principale d'une application.
    main

    /// L'élément HTML <nav> représente une section d'une page ayant des liens
    /// vers d'autres pages ou des fragments de cette page. Autrement dit,
    /// c'est une section destinée à la navigation dans un document (avec des
    /// menus, des tables des matières, des index, etc.).
    nav

    /// L'élément HTML <section> représente une section générique d'un
    /// document, par exemple un groupe de contenu thématique. Une section
    /// commence généralement avec un titre.
    section

    /// L'élément HTML <blockquote> (qui signifie bloc de citation) indique que
    /// le texte contenu dans l'élément est une citation longue. Le texte est
    /// généralement affiché avec une indentation (voir les notes ci-après).
    /// Une URL indiquant la source de la citation peut être donnée grâce à
    /// 'attribut cite tandis qu'un texte représentant la source peut être
    /// donné via l'élément <cite>.
    blockquote

    /// L'élément HTML <dd> fournit la description, la définition ou la valeur
    /// du terme précédent (<dt>) dans une liste de description (<dl>).
    dd

    /// L'élément HTML <div> (ou division) est le conteneur générique du
    /// contenu du flux. Il n'a aucun effet sur le contenu ou la mise en page
    /// tant qu'il n'est pas mis en forme d'une manière quelconque à l'aide
    /// de CSS.
    div

    /// L'élément HTML <dl> représente une liste de descriptions sous la forme
    /// d'une liste de paires associant des termes (fournis par des éléments
    /// <dt>) et leurs descriptions ou définitions (fournies par des éléments
    /// <dd>). On utilisera par exemple cet élément pour implémenter un
    /// glossaire.
    dl

    /// L'élément HTML <dt> identifie un terme dans une liste de définitions
    /// ou de descriptions. Cet élément n'apparaît qu'en tant qu'élément enfant
    /// d'un élément <dl> et est généralement suivi d'un élément <dd>.
    /// Plusieurs éléments <dt> qui se suivent indiqueront qu'ils partagent
    /// la définition/description fournie par le prochain élément <dd>.
    dt

    /// L'élément HTML <figcaption> représente une légende décrivant le reste
    /// du contenu de son élément parent <figure>.
    figcaption

    /// L'élément HTML <figure> représente un contenu autonome, éventuellement
    /// accompagné d'une légende facultative, qui est spécifiée à l'aide de
    /// l'élément <figcaption>. La figure, sa légende et son contenu sont
    /// référencés comme une seule unité.
    figure

    /// L'élément HTML <hr> représente un changement thématique entre des
    /// éléments de paragraphe (par exemple, un changement de décor dans un
    /// récit, un changement de sujet au sein d'une section).
    hr

    /// L'élément HTML <li> est utilisé pour représenter un élément dans
    /// liste. Il doit être contenu dans un élément parent : une liste
    /// ordonnée (ol), une liste non ordonnée (ul) ou un menu (menu). Dans les
    /// menus et les listes non ordonnées, les éléments de liste sont
    /// habituellement affichés en utilisant des puces. Dans les listes
    /// ordonnées, ils sont habituellement affichés avec compteur croissant
    /// à gauche, tel qu'un nombre ou une lettre.
    li

    /// L'élément HTML <menu> représente un groupe de commandes que
    /// l'utilisateur peut utiliser ou activer. Il peut être utilisé afin de
    /// créer des menus (affichés en haut d'un écran par exemple) et des
    /// menus contextuels (qui apparaissent au clic-droit ou après avoir
    /// cliqué sur un bouton).
    menu

    /// L'élément HTML <ol> représente une liste ordonnée. Les éléments d'une
    /// telle liste sont généralement affichés avec un indicateur ordinal
    /// pouvant prendre la forme de nombres, de lettres, de chiffres romains
    /// ou de points. La mise en forme de la numérotation n'est pas utilisée
    /// dans la description HTML mais dans la feuille de style CSS associée
    /// grâce à la propriété list-style-type.
    ol

    /// L'élément HTML <p> représente un paragraphe de texte. Les paragraphes
    /// sont généralement représentés comme des blocs et séparés par un espace
    /// vertical, leur première ligne est également parfois indentée.
    /// Les paragraphes sont des éléments blocs.
    p

    /// L'élément HTML <pre> représente du texte préformaté, généralement
    /// écrit avec une police à chasse fixe. Le texte est affiché tel quel,
    /// les espaces utilisés dans le document HTML seront retranscrits.
    pre

    /// L'élément HTML <ul> représente une liste d'éléments sans ordre
    /// particulier. Il est souvent représenté par une liste à puces.
    ul

    /// L'élément HTML <a> (pour ancre ou anchor en anglais), avec son attribut
    /// href, crée un lien hypertexte vers des pages web, des fichiers, des
    /// adresses e-mail, des emplacements se trouvant dans la même page, ou
    /// tout ce qu'une URL peut adresser. Le contenu de chaque élément <a>
    /// doit indiquer la destination du lien. Si l'attribut href est présent,
    /// appuyer sur la touche entrée en se concentrant sur l'élément <a>
    /// l'activera.
    a

    /// L'élément HTML <abbr> (abréviation en français) représente une
    /// abréviation ou un acronyme ; l'attribut facultatif title peut fournir
    /// une explication ou une description de l'abréviation. S'il est présent,
    /// title doit contenir cette description complète et rien d'autre.
    abbr

    /// L'élément HTML <b> permet d'attirer l'attention du lecteur sur un
    /// contenu qui n'a pas, pour autant, d'importance significative.
    /// Anciennement utilisé pour mettre le texte en gras. Cet élément ne doit
    /// pas être utilisé pour mettre en forme des éléments, c'est la propriété
    /// CSS font-weight qu'il faut utiliser. Si l'élément est d'une importance
    /// particulière, on utilisera l'élément HTML <strong>.
    b

    /// L'élément <bdi> (ou élément d'isolation de texte bidirectionnel) isole
    /// une portée (span) de texte pouvant être formatée dans une direction
    /// différente de celle du texte qui l'entoure. Cela permet, par exemple,
    /// de présenter correctement une citation en arabe (écrit de droite à
    /// gauche) au sein d'un texte écrit en français (écrit de gauche à droite).
    bdi

    /// L'élément HTML <bdo> (pour élément de remplacement du texte
    /// bidirectionnel) est utilisé afin d'outrepasser la direction du texte.
    /// Cela permet d'imposer une direction donnée à un texte. L'orientation
    /// du texte est inversée mais pas celle des caractères.
    bdo

    /// L'élément HTML <br> crée un saut de ligne (un retour chariot) dans le
    /// texte. Il s'avère utile lorsque les sauts de ligne ont une importance
    /// (par exemple lorsqu'on écrit une adresse ou un poème).
    br

    /// L'élément <cite> contient le titre d'une œuvre telle qu'un livre, une
    /// chanson, un film, une sculpture… Cet élément doit inclure le titre de
    /// l'œuvre. Cette référence peut-être abrégée en accord avec les
    /// conventions d'usages pour l'ajout des métadonnées de citations.
    cite

    /// L'élément HTML <code> représente un court fragment de code machine.
    /// Par défaut, l'agent utilisateur utilise une police à chasse fixe
    /// (monospace) afin d'afficher le texte contenu dans cet élément.
    code

    /// L'élément HTML <data> relie un contenu à une version de ce contenu
    /// interprétable par un ordinateur. Si le contenu possède une composante
    /// temporelle, l'élément <time> doit être utilisé.
    data

    /// L'élément HTML <dfn> (aussi nommé « définition ») est utilisé pour
    /// indiquer le terme défini dans le contexte d'une expression ou d'une
    /// phrase de définition. L'élément <p>, le couple <dt>/<dd> ou l'élément
    /// <section> qui est le plus proche ancêtre de <dfn> est considéré comme
    /// la définition du terme.
    dfn

    /// L'élément HTML <em> (pour emphase) est utilisé afin de marquer un
    /// texte sur lequel on veut insister. Les éléments <em> peuvent être
    /// imbriqués, chaque degré d'imbrication indiquant un degré d'insistance
    /// plus élevé.
    em

    /// L'élément HTML <i> représente un morceau de texte qui se différencie
    /// du texte principal. Cela peut par exemple être le cas pour des termes
    /// techniques, des phrases dans une langue étrangère ou encore
    /// l'expression des pensées d'un personnage. Le contenu de cet élément
    /// est généralement affiché en italique.
    i

    /// L'élément HTML <kbd> représente une plage de texte en ligne indiquant
    /// la saisie de texte par l'utilisateur à partir d'un clavier, d'une
    /// saisie vocale ou de tout autre dispositif de saisie de texte. Par
    /// convention, le user agent rend par défaut le contenu d'un élément
    /// <kbd> en utilisant sa police monospace, bien que cela ne soit pas
    /// requis par le standard HTML.
    kbd

    /// L'élément HTML <mark> représente un texte marqué ou surligné à cause
    /// de sa pertinence dans le contexte. Il peut par exemple être utilisé
    /// afin d'indiquer les correspondances d'un mot-clé recherché au sein
    /// d'un document.
    mark

    /// L'élément HTML <q> indique que le texte qu'il contient est une
    /// citation en incise. La plupart des navigateurs modernes entoure le
    /// text de cet élément avec des marques de citation. Cet élément est
    /// destiné aux citations courtes qui ne nécessitent pas de sauts de
    /// paragraphe. Pour les plus grandes citations, on utilisera l'élément
    /// blockquote.
    q

    /// L'élément HTML <rp> est utilisé pour fournir ce qui fera office de
    /// parenthèse aux navigateurs qui ne prennent pas en charge les
    /// annotations Ruby.
    rp

    /// L'élément HTML <rt> indique la composante texte d'une annotation Ruby,
    /// il est notamment utilisé pour la prononciation, la traduction ou la
    /// translitération des caractères d'Asie orientale. Cet élément est
    /// toujours contenu dans un élément ruby.
    rt

    /// L'élément HTML <ruby> représente une annotation ruby. Les annotations
    /// Ruby servent à afficher la prononciation des caractères d'Asie
    /// orientale.
    ruby

    /// L'élément HTML <s> permet d'afficher du texte qui est barré car il
    /// n'est plus pertinent ou car il est obsolète. <s> ne doit pas être
    /// employé pour indiquer des éditions dans un document (on utilisera
    /// alors del et ins).
    s

    /// L'élément HTML <samp> est un élément qui permet de représenter un
    /// résultat produit par un programme informatique. Il est généralement
    /// affiché avec la police à chasse fixe du navigateur (par exemple en
    /// Courier ou en Lucida Console).
    samp

    /// L'élément HTML <small> permet de représenter des commentaires ou des
    /// textes à écrire en petits caractères (des termes d'un contrat, des
    /// mentions relatives au droit d'auteur, etc.) quelle que soit la
    /// présentation.
    small

    /// L'élément HTML <span> est un conteneur générique en ligne (inline)
    /// pour les contenus phrasés. Il ne représente rien de particulier. Il
    /// peut être utilisé pour grouper des éléments afin de les mettre en
    /// forme (grâce aux attributs class ou id et aux règles CSS) ou parce
    /// qu'ils partagent certaines valeurs d'attribut comme lang. Il doit
    /// uniquement être utilisé lorsqu'aucun autre élément sémantique n'est
    /// approprié. <span> est très proche de l'élément div, mais l'élément
    /// <div> est un élément de bloc, alors que <span> est un élément en
    /// ligne.
    span

    /// L'élément HTML <strong> indique que le texte a une importance
    /// particulière ou un certain sérieux voire un caractère urgent. Cela se
    /// traduit généralement par un affichage en gras.
    strong

    /// L'élément HTML <sub> est utilisé, pour des raisons typographiques, afin
    /// d'afficher du texte souscrit (ou en indice) (plus bas et généralement
    /// plus petit) par rapport au bloc de texte environnant.
    sub

    /// L'élément HTML <sup> est utilisé, pour des raisons typographiques, afin
    /// d'afficher du texte en exposant (plus haut et généralement plus petit)
    /// par rapport au bloc de texte environnant.
    sup

    /// L'élément HTML <time> permet de représenter une période donnée. Cet
    /// élément permet d'utiliser l'attribut datetime afin de traduire la date
    /// ou l'instant dans un format informatique (permettant aux moteurs de
    /// recherche d'exploiter ces données ou de créer des rappels).
    time

    /// L'élément HTML <u> permet d'afficher un fragment de texte qui est
    /// annoté avec des éléments non textuels. Par défaut, le contenu de
    /// l'élément est souligné. Cela pourra par exemple être le cas pour
    /// marquer un texte comme étant un nom propre chinois, ou pour marquer
    /// un texte qui a été mal orthographié.
    u

    /// L'élément HTML <var> représente une variable dans une expression
    /// mathématique ou un texte lié à la programmation. Son contenu est
    /// généralement représenté avec une version italique de la police
    /// environnante utilisée, toutefois, ce comportement peut dépendre du
    /// navigateur utilisé.
    var

    /// L'élément HTML <wbr> permet de représenter un emplacement où casser
    /// la ligne si nécessaire. Le navigateur pourra alors utiliser cet
    /// emplacement pour effectuer un saut de ligne si le texte est trop long
    /// et qu'en temps normal, une règle empêche le saut de ligne.
    wbr

    /// L'élément HTML <area> définit une zone particulière d'une image et peut
    /// lui associer un lien hypertexte. Cet élément n'est utilisé qu'au sein
    /// d'un élément <map>.
    area

    /// L'élément HTML <audio> est utilisé afin d'intégrer un contenu sonore
    /// dans un document. Il peut contenir une ou plusieurs sources audio
    /// représentées avec l'attribut src ou l'élément <source> : le navigateur
    /// choisira celle qui convient le mieux. Il peut également être la
    /// destination de médias diffusés en continu, en utilisant un MediaStream.
    audio

    /// L'élément HTML <img> permet d'intégrer une image dans un document.
    img

    /// L'élément HTML <map> est utilisé avec des éléments area afin de définir
    /// une image cliquable divisée en régions.
    map

    /// L'élément HTML <track> est utilisé comme élément fils d'un élément
    /// <audio> ou <video> et permet de fournir une piste texte pour le média
    /// (par exemple afin de gérer automatiquement les sous-titres). Les pistes
    /// texte utilisées avec cet élément sont formatées selon le format WebVTT
    /// (ce sont des fichiers .vtt) (WebVTT pour Web Video Text Tracks).
    track

    /// L'élément HTML <video> intègre un contenu vidéo dans un document.
    video

    /// L'élément svg est un conteneur qui définit un nouveau système de
    /// coordonnées et une vue. Il est utilisé comme élément le plus externe
    /// des documents SVG, mais il peut également être utilisé pour intégrer
    /// un fragment SVG à l'intérieur d'un document SVG ou HTML.
    svg

    /// L'élément de niveau supérieur en MathML est <math>. Chaque instance
    /// MathML valide doit être enveloppée dans des balises <math>. En outre,
    /// vous ne devez pas imbriquer un deuxième élément <math> dans un autre,
    /// mais vous pouvez avoir un nombre arbitraire d'autres éléments enfants
    /// dans celui-ci.
    math

    /// On utilise l'élément HTML <canvas> avec l'API canvas, ou l'API WebGL
    /// pour dessiner des graphiques et des animations.
    canvas

    /// L'élément HTML <noscript> définit un fragment HTML qui doit être
    /// affiché si les fonctionnalités de script ne sont pas prises en charge
    /// ou si elles sont désactivées.
    noscript

    /// L'élément HTML <script> est utilisé pour intégrer ou faire référence à
    /// un script exécutable. Cela fait généralement référence à du code
    /// JavaScript mais ce peut également être un autre type de script (par
    /// exemple WebGL).
    script

    /// L'élément HTML <del> représente une portion de texte ayant été
    /// supprimée d'un document. Cet élément est souvent (mais pas
    /// nécessairement) affiché rayé. L'élément <ins> est quant à lui utilisé
    /// pour représenter des portions de texte ajoutées.
    del

    /// L'élément HTML <ins> représente un fragment de texte qui a été ajouté
    /// dans un document.
    ins

    /// L'élément <caption> définit la légende (ou le titre) d'un tableau.
    caption

    /// L'élément HTML <col> définit une colonne appartenant à un tableau et
    /// est utilisé afin de définir la sémantique commune à toutes ses cellules.
    /// On trouve généralement cet élément au sein d'un élément <colgroup>.
    col

    /// L'élément HTML <colgroup> définit un groupe de colonnes au sein d'un
    /// tableau.
    colgroup

    /// L'élément HTML <table> permet de représenter un tableau de données,
    /// c'est-à-dire des informations exprimées sur un tableau en deux
    /// dimensions.
    table

    /// L'élément HTML <tbody> permet de regrouper un ou plusieurs éléments
    /// tr afin de former le corps d'un tableau HTML (table).
    tbody

    /// L'élément HTML <td> définit une cellule d'un tableau qui contient des
    /// données. Cet élément fait partie du modèle de tableau.
    td

    /// L'élément HTML <tfoot> permet de définir un ensemble de lignes qui
    /// résument les colonnes d'un tableau.
    tfoot

    /// L'élément HTML <th> définit une cellule d'un tableau comme une cellule
    /// d'en-tête pour un groupe de cellule. La nature de ce groupe est définie
    /// grâce aux attributs scope et headers.
    th

    /// L'élément <thead> définit un ensemble de lignes qui définit l'en-tête
    /// des colonnes d'un tableau.
    thead

    /// L'élément HTML <tr> définit une ligne de cellules dans un tableau. Une
    /// ligne peut être constituée d'éléments <td> (les données des cellules)
    /// et <th> (les cellules d'en-têtes).
    tr

    /// L'élément <button> représente un bouton cliquable, utilisé pour
    /// soumettre des formulaires ou n'importe où dans un document pour une
    /// fonctionnalité de bouton accessible et standard. Par défaut, les
    /// boutons HTML sont présentés dans un style ressemblant à la plate-forme
    /// d'exécution de l'agent utilisateur, mais vous pouvez modifier
    /// l'apparence des boutons avec CSS.
    button

    /// L'élément HTML <datalist> contient un ensemble d'éléments <option> qui
    /// représentent les valeurs possibles pour d'autres contrôles.
    datalist

    /// L'élément HTML <fieldset> est utilisé afin de regrouper plusieurs
    /// contrôles interactifs ainsi que des étiquettes (<label>) dans un
    /// formulaire HTML.
    fieldset

    /// L'élément HTML <form> représente un formulaire, c'est-à-dire une
    /// section d'un document qui contient des contrôles interactifs
    /// permettant à un utilisateur de fournir des informations.
    form

    /// L'élément HTML <input> est utilisé pour créer un contrôle interactif
    /// dans un formulaire web qui permet à l'utilisateur de saisir des
    /// données. Les saisies possibles et le comportement de l'élément
    /// <input> dépend fortement de la valeur indiquée dans son attribut type.
    input

    /// L'élément HTML <label> représente une légende pour un objet d'une
    /// interface utilisateur. Il peut être associé à un contrôle en utilisant
    /// l'attribut for ou en plaçant l'élément du contrôle à l'intérieur de
    /// l'élément <label>. Un tel contrôle est appelé contrôle étiqueté par
    /// l'élément <label>.
    label

    /// L'élément HTML <legend> représente une légende pour le contenu de son
    /// élément parent fieldset.
    legend

    /// L'élément HTML <meter> représente une valeur scalaire dans un
    /// intervalle donné ou une valeur fractionnaire.
    meter

    /// L'élément HTML <optgroup>, utilisé dans un formulaire, permet de créer
    /// un groupe d'options parmi lesquelles on peut choisir dans un élément
    /// select.
    optgroup

    /// L'élément HTML <option>, utilisé dans un formulaire, permet de
    /// représenter un contrôle au sein d'un élément select, optgroup ou
    /// datalist. Cet élément peut donc représenter des éléments d'un menu
    /// dans un document HTML.
    option

    /// L'élément HTML <output> représente un conteneur dans lequel un site ou
    /// une application peut injecter le résultat d'un calcul ou d'une action
    /// utilisateur.
    output

    /// L'élément HTML <progress> indique l'état de complétion d'une tâche et
    /// est généralement représenté par une barre de progression.
    progress

    /// L'élément HTML <select> représente un contrôle qui fournit une liste
    /// d'options parmi lesquelles l'utilisateur pourra choisir.
    select

    /// L'élément HTML <textarea> représente un contrôle qui permet d'éditer
    /// du texte sur plusieurs lignes.
    textarea

    /// L'élément HTML <details> est utilisé comme un outil permettant de
    /// révéler une information. Un résumé ou un intitulé peuvent être fournis
    /// grâce à un élément <summary>.
    details

    /// L'élément HTML <dialog> représente une boite de dialogue ou un
    /// composant interactif (par exemple un inspecteur ou une fenêtre).
    dialog

    /// L'élément HTML <summary> représente une boîte permettant de révéler le
    /// contenu d'un résumé ou d'une légende pour le contenu d'un élément
    /// details. En cliquant sur l'élément <summary>, on passe de l'état
    /// affiché à l'état masqué (et vice versa) de l'élément <details> parent.
    summary

    /// L'élément HTML <slot> représente un emplacement d'un composant web
    /// qu'on peut remplir avec son propre balisage. On peut ainsi obtenir un
    /// document construit avec différents arbres DOM. Cet élément fait partie
    /// des outils relatifs aux composants web (Web Components).
    slot

    /// L'élément HTML <template> (ou Template Content ou modèle de contenu)
    /// est un mécanisme utilisé pour stocker du contenu HTML (côté client)
    /// qui ne doit pas être affiché lors du chargement de la page mais qui
    /// peut être instancié et affiché par la suite grâce à un script
    /// JavaScript.
    template

    /// L'élément HTML <acronym>, pour les acronymes, permet aux auteurs de
    /// pages d'indiquer une suite de caractères composant un acronyme ou
    /// l'abréviation d'un mot.
    #[deprecated = "Élément obsolète ou déprécié"]
    acronym

    /// L'élément <applet>, pour les applets, définit l'intégration d'un
    /// applet Java.
    #[deprecated = "Cet élément est désormais déprécié en faveur de <object>"]
    applet

    /// L'élément HTML <basefont> définit la police par défaut (taille, fonte,
    /// couleur) pour les éléments qui sont des descendants de cet élément.
    /// La taille de la police utilisée peut ensuite varier relativement à
    /// cette taille de base grâce à l'élément <font>.
    #[deprecated = "Élément obsolète ou déprécié"]
    basefont

    /// L'élément HTML <bgsound> (pour background sound ou « son d'arrière-plan
    /// ») est un élément défini par Internet Explorer qui permet d'associer
    /// un son d'ambiance à une page.
    #[deprecated = "Élément obsolète ou déprécié"]
    bgsound

    /// L'élément HTML <big> (gros) augmente d'une taille la police du texte de
    /// l'élément (il permet par exemple de passer de small à medium, ou de
    /// large à x-large) jusqu'à atteindre la taille maximale autorisée
    /// par le navigateur.
    #[deprecated = "Élément obsolète ou déprécié"]
    big

    /// L'élément HTML <blink> (N.D.T le verbe blink signifie « clignoter »)
    /// est un élément non-standard faisant clignoter le texte qu'il contient.
    #[deprecated = "Élément obsolète ou déprécié"]
    blink

    /// L'élément <center> est un élément de bloc qui contient des paragraphes
    /// et d'autres éléments de type bloc ou en ligne. Le contenu entier de cet
    /// élément est centré horizontalement au sein de son conteneur parent
    /// (généralement l'élément <body>).
    #[deprecated = "Élément obsolète ou déprécié"]
    center

    /// L'élément HTML <content> — une partie obsolète de la suite de
    /// technologies Web Components — était utilisé à l'intérieur de Shadow
    /// DOM comme un point d'insertion, et n'était pas destiné à être utilisé
    /// dans du HTML ordinaire.
    #[deprecated = "Il a maintenant été remplacé par l'élément <slot>, qui crée un point dans le DOM où un Shadow DOM peut être inséré."]
    content

    /// L'élément HTML <dir> (pour directory) est utilisé comme un conteneur
    /// pour un répertoire (c'est-à-dire un ensemble de fichiers). Des styles
    /// et icônes peuvent être appliqués par l'agent utilisateur. Cet élément
    /// obsolète ne doit pas être utilisé, il peut être remplacé par l'élément
    /// <ul> qui permet de représenter des listes et, entre autres, des listes
    /// de fichiers.
    #[deprecated = "Utiliser <ul> à la place"]
    dir

    /// L'élément HTML <font> définit la taille, la couleur et la police
    /// de son contenu.
    #[deprecated = "Élément obsolète ou déprécié"]
    font

    /// L'élément HTML <frame> définit une zone particulière dans laquelle un
    /// autre document HTML est affiché. Une <frame> (un « cadre » en
    /// français) doit être utilisée dans un élément <frameset>.
    #[deprecated = "Élément obsolète ou déprécié"]
    frame

    /// L'élément HTML <frameset> est utilisé pour contenir les éléments
    /// <frame>.
    #[deprecated = "Élément obsolète ou déprécié"]
    frameset

    /// L'élément HTML <hgroup> représente un titre de plusieurs niveaux pour
    /// une section d'un document. Il regroupe un ensemble d'éléments
    /// <h1>–<h6>.
    #[deprecated = "Élément obsolète ou déprécié"]
    hgroup

    /// L'élément HTML <image> est un élément obsolète, remplacé depuis
    /// longtemps par l'élément standard img.
    #[deprecated = "Élément obsolète ou déprécié"]
    image

    /// L'élément HTML <keygen> existe afin de faciliter la génération de clés
    /// et l'envoi d'une clé publique via un formulaire HTML. Le mécanisme
    /// utilisé est conçu pour être utilisé avec les systèmes de gestion de
    /// certificats électroniques. L'élément keygen est prévu pour être utilisé
    /// dans un formulaire HTML avec d'autres informations permettant de
    /// construire une requête de certificat, le résultat du processus étant
    /// un certificat signé.
    #[deprecated = "Élément obsolète ou déprécié"]
    keygen

    /// L'élément HTML <marquee> est utilisé pour insérer une zone de texte
    /// défilant.
    #[deprecated = "Élément obsolète ou déprécié"]
    marquee

    /// L'élément HTML <menuitem> représente une commande qu'un utilisateur
    /// peut utiliser via un menu contextuel ou un menu rattaché à un bouton.
    #[deprecated = "Élément obsolète ou déprécié"]
    menuitem

    /// L'élément HTML <nobr> évite qu'un texte soit coupé par un retour à la
    /// ligne automatique ; il est donc affiché sur une seule ligne. Il peut
    /// être alors nécessaire d'utiliser les barres de défilement pour lire
    /// le texte en intégralité.
    #[deprecated = "Élément obsolète ou déprécié"]
    nobr

    /// L'élément <noembed> est une façon obsolète et non standardisée de
    /// fournir une alternative de contenu pour les navigateurs ne supportant
    /// pas l'élément embed ou des catégories de contenu qu'un auteur aimerait
    /// utiliser. Cet élément a été rendu obsolète à partir de la version
    /// HTML 4.01 et a été remplacé par object. Le contenu alternatif doit
    /// être inséré entre la balise d'ouverture et celle de fermeture de
    /// object.
    #[deprecated = "Élément obsolète ou déprécié"]
    noembed

    /// L'élément HTML obsolète <noframes> est utilisé par les navigateurs qui
    /// ne supportent pas les éléments frame, ou qui sont configurés afin de
    /// ne pas les supporter.
    #[deprecated = "Élément obsolète ou déprécié"]
    noframes

    /// L'élément HTML <param> définit les paramètres qui peuvent être employés
    /// dans un élément object.
    #[deprecated = "Élément obsolète ou déprécié"]
    param

    /// L'élément HTML <plaintext> permet d'afficher du texte qui n'est pas
    /// interprété comme du HTML. Il ne possède pas de balise de fermeture,
    /// car tout ce qui suit n'est plus considéré comme du HTML.
    #[deprecated = "Élément obsolète ou déprécié"]
    plaintext

    /// L'élément de base ruby (<rb>) est utilisé afin de délimiter le
    /// composant texte de base d'une annotation ruby. Autrement dit, le
    /// texte qui est annoté. Un élément <rb> devrait encadrer chaque segment
    /// atomique du texte de base.
    #[deprecated = "Élément obsolète ou déprécié"]
    rb

    /// L'élément <rtc> permet d'ajouter des notations Ruby sémantiques. Il est
    /// donc « proche » des éléments liées à la représentation Ruby comme
    /// rb, ruby. Les éléments rb peuvent être annotés pour la prononciation
    /// (rt) ou pour la sémantique (rtc).
    #[deprecated = "Élément obsolète ou déprécié"]
    rtc

    /// L'élément HTML <shadow> était utilisé comme un point d'insertion
    /// (insertion point) du shadow DOM. Cet élément a été retiré de la
    /// spécification et est désormais obsolète.
    #[deprecated = "Élément obsolète ou déprécié"]
    shadow

    /// L'élément HTML <spacer> était utilisé pour insérer des blancs au sein
    /// d'une page web. Il a été créé par Netscape pour obtenir le même effet
    /// que celui qui était créé avec des images GIF d'un pixel, permettant
    /// d'ajouter des espaces blancs. Cependant, <spacer> n'est pas pris en
    /// charge par les principaux navigateurs principaux et il faut utiliser
    /// les règles CSS pour obtenir ces effets d'alignement
    #[deprecated = "Élément obsolète ou déprécié"]
    spacer

    /// L'élément HTML <strike> permet de représenter du texte barré ou avec
    /// une ligne le traversant.
    #[deprecated = "Élément obsolète ou déprécié"]
    strike

    /// L'élément HTML <tt> (pour Teletype Text) crée un élément en ligne,
    /// écrit dans la police à chasse fixe par défaut du navigateur.
    /// Cet élément a été conçu pour mettre en forme du texte comme s'il
    /// apparaissait sur un affichage à largeur fixe tel qu'un téléscripteur.
    #[deprecated = "Élément obsolète ou déprécié"]
    tt

    /// L'élément HTML <xmp> (pour example) affiche le texte entre les balises
    /// d'ouverture et de fermeture sans interpréter le HTML qu'il contient et
    /// en utilisant une police à chasse fixe. La spécification HTML 2
    /// recommande un affichage suffisamment large pour contenir 80
    /// caractères par ligne.
    #[deprecated = "Élément obsolète ou déprécié"]
    xmp

    #[deprecated = "Élément obsolète ou déprécié"]
    listing

    /// L'élément HTML <embed> permet d'intégrer du contenu externe à cet
    /// endroit dans le document. Le contenu peut être fourni par une
    /// application externe ou une autre source telle qu'un plugin du
    /// navigateur.
    embed

    /// L'élément HTML <iframe> représente un contexte de navigation imbriqué
    /// qui permet en fait d'obtenir une page HTML intégrée dans la page
    /// courante.
    iframe

    /// L'élément HTML <object> représente une ressource externe qui peut
    /// être interprétée comme une image, un contexte de navigation imbriqué
    /// ou une ressource à traiter comme un plugin.
    object

    /// L'élément HTML <source> définit différentes ressources média pour un
    /// élément <picture>, <audio> ou <video>. C'est un élément vide : il ne
    /// possède pas de contenu et ne nécessite pas de balise fermante. Il est
    /// généralement utilisé pour distribuer le même contenu en utilisant les
    /// différents formats pris en charge par les différents navigateurs.
    source

    /// L'élément HTML <picture> est un conteneur utilisé afin de définir zéro
    /// ou plusieurs éléments <source> destinés à un élément <img>. Le
    /// navigateur choisira la source la plus pertinente selon la disposition
    /// de la page (les contraintes qui s'appliquent à la boîte dans laquelle
    /// l'image devra être affichée), selon l'appareil utilisé (la densité de
    /// pixels de l'affichage par exemple avec les appareils hiDPI) et selon
    /// les formats pris en charge (ex. WebP pour les navigateurs Chromium ou
    /// PNG pour les autres). Si aucune correspondance n'est trouvée parmi les
    /// éléments <source>, c'est le fichier défini par l'attribut src de
    /// l'élément <img> qui sera utilisé.
    picture

    /// L'élément HTML <portal> permet d'embarquer une autre page HTML à
    /// l'intérieur de la page courante afin de permettre une navigation plus
    /// souple vers de nouvelles pages.
    portal

    /* MathML */
    // math
    maction
    maligngroup
    malignmark
    menclose
    merror
    mfenced
    mfrac
    mglyph
    mi
    mlabeledtr
    mlongdiv
    mmultiscripts
    mn
    mo
    mover
    mpadded
    mphantom
    mroot
    mrow
    ms
    mscarries
    mscarry
    msgroup
    msline
    mspace
    msqrt
    msrow
    mstack
    mstyle
    msub
    msup
    msubsup
    mtable
    mtd
    mtext
    mtr
    munder
    munderover
    semantics
    annotation
    annotationXml

    /* SVG */

    // a
    animate
    animateMotion
    animateTransform
    circle
    clipPath
    colorProfile
    defs
    desc
    discard
    ellipse
    feBlend
    feColorMatrix
    feComponentTransfer
    feComposite
    feConvolveMatrix
    feDiffuseLighting
    feDisplacementMap
    feDistantLight
    feDropShadow
    feFlood
    feFuncA
    feFuncB
    feFuncG
    feFuncR
    feGaussianBlur
    feImage
    feMerge
    feMergeNode
    feMorphology
    feOffset
    fePointLight
    feSpecularLighting
    feSpotLight
    feTile
    feTurbulence
    filter
    foreignObject
    g
    hatch
    hatchpath
    // image
    line
    linearGradient
    marker
    mask
    mesh
    meshgradient
    meshpatch
    meshrow
    metadata
    mpath
    path
    pattern
    polygon
    polyline
    radialGradient
    rect
    // script
    set
    solidcolor
    stop
    // style
    // svg
    switch
    symbol
    text
    textPath
    // title
    tspan
    unknown
    r#use
    view

    customElement
}
