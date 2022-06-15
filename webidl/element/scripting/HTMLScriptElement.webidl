[Exposed=Window]
interface HTMLScriptElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute USVString src;
  [CEReactions] attribute DOMString type;
  [CEReactions] attribute boolean noModule;
  [CEReactions] attribute boolean async;
  [CEReactions] attribute boolean defer;
  [CEReactions] attribute DOMString? crossOrigin;
  [CEReactions] attribute DOMString text;
  [CEReactions] attribute DOMString integrity;
  [CEReactions] attribute DOMString referrerPolicy;
  [SameObject, PutForwards=value] readonly attribute DOMTokenList blocking;

  static boolean supports(DOMString type);

  // also has obsolete members
};
