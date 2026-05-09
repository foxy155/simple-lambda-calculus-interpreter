use std::io::{self, stdin};
use std::collections::HashMap;
use std::sync::LazyLock;


pub static TOKEN_MAP: LazyLock<HashMap<char, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('(', "LPAREN");
    m.insert(')', "RPAREN");
    m.insert('.', "DOT");
    m.insert('λ', "LAMBDA");
    m
});

#[derive(Debug,Clone)]
pub enum Expr{
    Var(String),
    Abs(String, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
}

fn main() -> io::Result<()> {
    println!("enter lambda calculus expression:");
    let mut lambda = String::new();
    stdin().read_line(&mut lambda)?;
    let lambda = lambda.trim().to_string();
    println!("lambda: {}", lambda);

    println!("\n-----choose option------");
    println!("1: parse inputted string");
    println!("2: apply reduction to the inputted string");
    println!("3: other");

    let mut choice = String::new();
    stdin().read_line(&mut choice)?;
    let choice = choice.trim().parse::<i8>().unwrap_or(0);
    let tokens = lexer(&lambda);
    let ast = parser(tokens.clone());

    match choice {
        1 => {

            println!("\nCollected tokens: {:?}", tokens);
            println!("\n----------");
            println!("{:?}", ast);
        }
        2 => {
            let reduced = reduce(ast);
            println!("\nreduced reduction: {:?}", reduced);
        }

        _ => println!("wrong input"),
    }

    Ok(())
}


fn lexer(input: &str) -> Vec<(&'static str, char)> {
    let mut out: Vec<(&'static str, char)> = Vec::new();
    let mut right_p: i8 = 0;
    let mut left_p: i8 = 0;
    for (line_idx, line) in input.lines().enumerate() {
        println!("this is line {}", line_idx + 1);
        println!("parsing line {}", line_idx + 1);

        for ch in line.chars() {
            println!("{}", ch);

            if ch.is_whitespace() {
                continue;
            }

            if let Some(token_name) = TOKEN_MAP.get(&ch) {
                println!("{},{}", token_name, ch);
                out.push((*token_name, ch));
                if ch == '(' {
                    left_p = left_p + 1;
                }
                if ch == ')' {
                    right_p = right_p + 1;
                }
                continue;
            }

            if ch.is_alphabetic() {
                println!("variable,{}", ch);
                out.push(("VARIABLE", ch));
                continue;
            }


            println!("there is a bad character in this lambda: {}", ch);
        }
    }
    if right_p != left_p{panic!("incorrect amount of parenthases");}



    out
}

fn substitute(expr: &Expr, var: &str, replacement: &Expr) -> Expr {
    match expr {
        Expr::Var(name) => {
            if name == var {
                replacement.clone()
            } else {
                Expr::Var(name.clone())
            }
        }

        Expr::Abs(param, body) => {
            if param == var {
                Expr::Abs(param.clone(), body.clone())
            } else {
                Expr::Abs(
                    param.clone(),
                    Box::new(substitute(body, var, replacement)),
                )
            }
        }

        Expr::App(func, arg) => {
            Expr::App(
                Box::new(substitute(func, var, replacement)),
                Box::new(substitute(arg, var, replacement)),
            )
        }
    }
}

fn reduce_once(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::App(func, arg) => {
            match &**func {
                Expr::Abs(param, body) => {
                    let result = substitute(body, param, arg);
                    Some(result)
                }
                _ => {
                    if let Some(new_func) = reduce_once(func) {
                        Some(Expr::App(Box::new(new_func), arg.clone()))
                    } else if let Some(new_arg) = reduce_once(arg) {
                        Some(Expr::App(func.clone(), Box::new(new_arg)))
                    } else {
                        None
                    }
                }
            }
        }

        Expr::Abs(param, body) => {
            if let Some(new_body) = reduce_once(body) {
                Some(Expr::Abs(param.clone(), Box::new(new_body)))
            } else {
                None
            }
        }


        Expr::Var(_) => None,
    }
}


fn reduce(mut expr: Expr) -> Expr {
    while let Some(next) = reduce_once(&expr) {
        expr = next;
    }
    expr
}


fn parser(input: Vec<(&'static str, char)>) -> Expr {
    let mut p = Parser::new(input);
    p.parse_expr()
}

struct Parser {
    tokens: Vec<(&'static str, char)>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<(&'static str, char)>) -> Self {
        Self {tokens, pos: 0}
    }
    fn peek(&self) -> Option<&(&'static str, char)> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<(&'static str, char)> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn parse_expr(&mut self) -> Expr {
        let mut expr = self.parse_atom();

        while let Some((tok, _)) = self.peek() {
            if *tok == "VARIABLE" || *tok == "LAMBDA" || *tok == "LPAREN" {
                let right = self.parse_atom();
                expr = Expr::App(Box::new(expr), Box::new(right));
            } else {
                break;
            }
        }

        expr
    }
    fn parse_atom(&mut self) -> Expr {
        match self.peek().cloned() {
            Some(("VARIABLE", ch)) => {
                self.next();
                Expr::Var(ch.to_string())
            }

            Some(("LAMBDA", _)) => self.parse_lambda(),

            Some(("LPAREN", _)) => {
                self.next();
                let expr = self.parse_expr();
                match self.next() {
                    Some(("RPAREN", _)) => expr,
                    _ => panic!("expected ')'"),
                }
            }

            other => panic!("unexpected token: {:?}", other),
        }
    }
    fn parse_lambda(&mut self) -> Expr {
        self.next();

        let param = match self.next() {
            Some(("VARIABLE", ch)) => ch.to_string(),
            _ => panic!("expected variable after λ"),
        };

        match self.next() {
            Some(("DOT", _)) => {}
            _ => panic!("expected '.' after lambda parameter"),
        }

        let body = self.parse_expr();
        Expr::Abs(param, Box::new(body))
    }
}
