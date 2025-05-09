import {
    LRLanguage,
    LanguageSupport,
    delimitedIndent,
    continuedIndent,
    indentNodeProp,
} from '@codemirror/language';
import { parser } from './syntax.grammar';
import { styleTags, tags as t } from '@lezer/highlight';

export const wgslLanguage = LRLanguage.define({
    name: 'wgsl',
    parser: parser.configure({
        props: [
            indentNodeProp.add({
                ifStatement: continuedIndent({ except: /^\s*({|else\b)/ }),
                CompoundStatement: delimitedIndent({ closing: '}' }),
                StructBodyDeclaration: delimitedIndent({ closing: '}' }),
            }),
            styleTags({
                Assign: t.operator,
                AddAssign: t.operator,
                SubAssign: t.operator,
                MulAssign: t.operator,
                DivAssign: t.operator,
                ModAssign: t.operator,
                LeftAssign: t.operator,
                RightAssign: t.operator,
                AndAssign: t.operator,
                XorAssign: t.operator,
                OrAssign: t.operator,
                Add: t.operator,
                Sub: t.operator,
                Mul: t.operator,
                Div: t.operator,
                Mod: t.operator,
                Left: t.operator,
                Right: t.operator,
                And: t.operator,
                Xor: t.operator,
                Or: t.operator,
                AndAnd: t.operator,
                OrOr: t.operator,
                Inc: t.operator,
                Dec: t.operator,
                Bang: t.operator,
                Tilde: t.operator,
                Eq: t.operator,
                Neq: t.operator,
                Lt: t.operator,
                Lte: t.operator,
                Gt: t.operator,
                Gte: t.operator,
                '<': t.operator,
                '>': t.operator,
                ReturnType: t.operator,

                Comment: t.comment,
                LineComment: t.comment,
                BlockComment: t.comment,
                'FunctionHeader/Identifier': t.macroName,
                'FunctionCall/Identifier': t.macroName,

                Keyword: t.keyword,
                Type: t.typeName,
                TypeDeclaration: t.typeName,
                Attribute: t.attributeName,
                'Attribute/Identifier': t.attributeName,
                'Attribute/IntLiteral': t.number,
                IntLiteral: t.number,
                UintLiteral: t.number,
                FloatLiteral: t.number,
                String: t.string,
                true: t.number,
                false: t.number,
                Directive: t.keyword,
                Identifier: t.macroName,
            }),
        ],
    }),
    languageData: {
        closeBrackets: { brackets: ['(', '[', '{', "'", '"', '`'] },
        commentTokens: { line: '//', block: { open: '/*', close: '*/' } },
        indentOnInput: /^\s*(?:case |default:|\{|\}|<\/)$/,
        wordChars: '$',
    },
});

export function wgsl() {
    return new LanguageSupport(wgslLanguage);
}
