use std::borrow::Cow;
use std::fmt;

use ::analyzer::MorphAnalyzer;
use ::container::abc::*;
use ::container::Lex;
use ::container::HyphenSeparatedParticle;
use ::container::Score;
use ::container::stack::Stack;
use ::container::stack::StackAffix;
use ::container::stack::StackHyphenated;
use ::container::stack::StackSource;
use ::opencorpora::OpencorporaTagReg;

use container::decode::*;


#[derive(Debug, Clone, PartialEq)]
pub struct StackParticle {
    pub stack: StackHyphenated,
    pub particle: Option<HyphenSeparatedParticle>,
}


impl From<StackHyphenated> for StackParticle {
    fn from(stack: StackHyphenated) -> Self { StackParticle { stack: stack, particle: None } }
}

impl From<StackAffix> for StackParticle {
    fn from(stack: StackAffix) -> Self { StackHyphenated::from(stack).into() }
}

impl From<StackSource> for StackParticle {
    fn from(stack: StackSource) -> Self { StackAffix::from(stack).into() }
}


impl Source for StackParticle {
    fn score(&self) -> Score {
        unimplemented!()
    }

    fn is_lemma(&self) -> bool {
        self.stack.is_lemma()
    }

    fn is_known(&self) -> bool {
        self.stack.is_known()
    }

    fn get_word(&self) -> Cow<str> {
        match self.particle {
            None                => self.stack.get_word(),
            Some(ref particle)  => {
                Cow::from(format!("{}{}", self.stack.get_word(), particle.particle))
            }
        }
    }

    fn get_normal_form(&self, morph: &MorphAnalyzer) -> Cow<str> {
        match self.particle {
            None                => self.stack.get_normal_form(morph),
            Some(ref particle)  =>
                format!("{}{}", self.stack.get_normal_form(morph), particle.particle).into(),
        }
    }
    fn get_tag<'m>(&self, morph: &'m MorphAnalyzer) -> &'m OpencorporaTagReg {
        self.stack.get_tag(morph)
    }

    fn try_get_para_id(&self) -> Option<u16> {
        self.stack.try_get_para_id()
    }

    fn write_word<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        self.stack.write_word(f)?;
        if let Some(ref particle) = self.particle {
            write!(f, "{}", particle.particle)?;
        }
        Ok(())
    }

    fn write_normal_form<W: fmt::Write>(&self, f: &mut W, morph: &MorphAnalyzer) -> fmt::Result {
        self.stack.write_normal_form(f, morph)?;
        if let Some(ref particle) = self.particle {
            write!(f, "{}", particle.particle)?;
        }
        Ok(())
    }

    fn get_lexeme(&self, morph: &MorphAnalyzer) -> Vec<Lex> {
        self.iter_lexeme(morph).collect()
    }

    fn get_lemma(&self, morph: &MorphAnalyzer) -> Lex {
        self.iter_lexeme(morph).next().unwrap()
    }
}


impl StackParticle {
    pub fn iter_lexeme<'s: 'i, 'm: 'i, 'i>(&'s self, morph: &'m MorphAnalyzer) -> impl Iterator<Item = Lex> + 'i {
        self.stack.iter_lexeme(morph).map(move |lex: Lex| Lex {
            stack: StackParticle {
                stack: match lex.stack {
                    Stack::Hyphenated(stack) => stack,
                    _ => unreachable!()
                },
                particle: self.particle.clone()
            }.into()
        } )
    }
}


impl MorphySerde for StackParticle {
    fn encode<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        self.stack.encode(f)?;
        if let Some(ref particle) = self.particle {
            write!(f, ";hp:{}", particle.particle)?;
        }
        Ok(())
    }

    fn decode(s: &str) -> Result<(&str, Self), DecodeError> {
        let (s, stack) = StackHyphenated::decode(s)?;
        let mut result = (s, StackParticle {
            stack: stack,
            particle: None,
        });
        if !s.is_empty() {
            match (|s| {
                let s = follow_str(s, ";")?;
                let s = follow_str(s, "hp").map_err(|e| match e {
                    DecodeError::DoesntMatch => DecodeError::UnknownPartType,
                    _ => e,
                })?;
                let (s, word) = take_str_until_char_is(follow_str(s, ":")?, ';')?;
                Ok((s, HyphenSeparatedParticle {
                    particle: word.to_string(),
                }))
            })(s) {
                Err(DecodeError::UnknownPartType) => (),
                Err(e) => Err(e)?,
                Ok((s, particle)) => {
                    result.0 = s;
                    result.1.particle = Some(particle);
                },
            };
        }
        Ok(result)
    }
}
