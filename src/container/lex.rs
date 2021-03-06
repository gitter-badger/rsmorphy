use std::borrow::Cow;
use std::fmt;

use analyzer::MorphAnalyzer;
use container::Score;
use container::Seen;
use container::stack::Stack;
use container::abc::*;
use opencorpora::OpencorporaTagReg;
use opencorpora::GrammemeSet;

use container::decode::*;

pub type Lexeme = Vec<Lex>;


#[derive(Debug, Clone, PartialEq)]
pub struct Lex {
    pub stack: Stack,
}


impl Lex {
    pub fn from_id<'id, S: 'id>(_morph: &MorphAnalyzer, id: S) -> Result<Self, DecodeError> where S: Into<&'id str> {
        Self::decode(id.into()).map(|(_, lex)| Ok(lex))?
    }

    pub fn from_stack<S>(_morph: &MorphAnalyzer, stack: S) -> Self where S: Into<Stack> {
        Lex { stack: stack.into() }
    }

    pub fn iter_lexeme<'s: 'i, 'm: 'i, 'i>(&'s self, morph: &'m MorphAnalyzer) -> impl Iterator<Item = Lex> + 'i {
        self.stack.iter_lexeme(morph)
    }

    pub fn as_seen<'m>(&'m self, morph: &'m MorphAnalyzer) -> Seen<'m> {
        Seen {
            word: self.get_word(),
            tag: self.get_tag(morph),
            para_id: self.stack.try_get_para_id(),
        }
    }

    pub fn get_plural<'m>(&self, morph: &'m MorphAnalyzer, num: usize) -> Option<Lex> {
        self.inflect(morph, &self.get_tag(morph).numeral_agreement_grammemes(num))
    }

    pub fn inflect<'m>(&self, morph: &MorphAnalyzer, required: &GrammemeSet) -> Option<Lex> {
        let new_grammemes = self.get_tag(morph).prepare_required(morph, required);
        self.iter_lexeme(morph)
            .map(|lex| {
                let hsl = lex.get_tag(morph).grammemes.set.intersection(&new_grammemes.set).count();
                (lex, hsl)
            })
            .max_by_key(|&(_, hsl)| hsl)
            .map(|(lex, _)| lex)

    }
}


impl Source for Lex {
    fn score(&self) -> Score {
        self.stack.score()
    }

    fn is_lemma(&self) -> bool {
        self.stack.is_lemma()
    }

    fn is_known(&self) -> bool {
        self.stack.is_known()
    }

    fn get_word(&self) -> Cow<str> {
        self.stack.get_word()
    }

    fn get_normal_form(&self, morph: &MorphAnalyzer) -> Cow<str> {
        self.stack.get_normal_form(morph)
    }

    fn get_tag<'m>(&self, morph: &'m MorphAnalyzer) -> &'m OpencorporaTagReg {
        self.stack.get_tag(morph)
    }

    fn try_get_para_id(&self) -> Option<u16> {
        self.stack.try_get_para_id()
    }

    fn write_word<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        self.stack.write_word(f)
    }

    fn write_normal_form<W: fmt::Write>(&self, f: &mut W, morph: &MorphAnalyzer) -> fmt::Result {
        self.stack.write_normal_form(f, morph)
    }

    fn get_lexeme(&self, morph: &MorphAnalyzer) -> Vec<Lex> {
        self.stack.get_lexeme(morph)
    }

    fn get_lemma(&self, morph: &MorphAnalyzer) -> Lex {
        self.stack.get_lemma(morph)
    }
}


impl fmt::Display for Lex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_word(f)
    }
}


impl MorphySerde for Lex {
    fn encode<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        write!(f, "ru:")?;
        self.stack.encode(f)
    }

    fn decode(s: &str) -> Result<(&str, Self), DecodeError> {
        let s = follow_str(s, "ru").map_err(|_| DecodeError::UnknownPartType)?;
        let (s, stack) = Stack::decode(follow_str(s, ":")?)?;
        Ok( (s, Lex { stack: stack }))
    }
}
