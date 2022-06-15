[Exposed=Window]
interface HTMLMetaElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute DOMString name;
  [CEReactions] attribute DOMString httpEquiv;
  [CEReactions] attribute DOMString content;
  [CEReactions] attribute DOMString media;

  // also has obsolete members
};
