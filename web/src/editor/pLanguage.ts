// Syntax highlighting for P (a stream tokenizer; no parse tree needed for a demo).
import { StreamLanguage } from "@codemirror/language";
import { pTokenizer } from "./pTokens";

export const pLanguage = StreamLanguage.define(pTokenizer);
