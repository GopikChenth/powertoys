/// Self-contained, zero-dependency math evaluator for PowerToys Run
pub fn evaluate_math(query: &str) -> Option<f64> {
    let clean: String = query.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.is_empty() {
        return None;
    }

    // Must contain math operators
    if !clean.chars().any(|c| "+-*/^".contains(c)) {
        return None;
    }

    if !clean.chars().all(|c| c.is_ascii_digit() || "+-*/.^()".contains(c)) {
        return None;
    }

    let mut parser = Parser::new(&clean);
    parser.parse_expression().filter(|_| parser.pos == parser.tokens.len())
}

struct Parser {
    tokens: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            tokens: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.tokens.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn parse_expression(&mut self) -> Option<f64> {
        let mut value = self.parse_term()?;
        while let Some(op) = self.peek() {
            if op == '+' {
                self.next();
                value += self.parse_term()?;
            } else if op == '-' {
                self.next();
                value -= self.parse_term()?;
            } else {
                break;
            }
        }
        Some(value)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut value = self.parse_factor()?;
        while let Some(op) = self.peek() {
            if op == '*' {
                self.next();
                value *= self.parse_factor()?;
            } else if op == '/' {
                self.next();
                let denom = self.parse_factor()?;
                if denom == 0.0 {
                    return None;
                }
                value /= denom;
            } else {
                break;
            }
        }
        Some(value)
    }

    fn parse_factor(&mut self) -> Option<f64> {
        let mut value = self.parse_primary()?;
        while let Some(op) = self.peek() {
            if op == '^' {
                self.next();
                let exp = self.parse_primary()?;
                value = value.powf(exp);
            } else {
                break;
            }
        }
        Some(value)
    }

    fn parse_primary(&mut self) -> Option<f64> {
        if let Some('(') = self.peek() {
            self.next();
            let val = self.parse_expression()?;
            if self.next() != Some(')') {
                return None;
            }
            return Some(val);
        }

        let mut num_str = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                num_str.push(c);
                self.next();
            } else {
                break;
            }
        }

        if num_str.is_empty() {
            None
        } else {
            num_str.parse::<f64>().ok()
        }
    }
}
