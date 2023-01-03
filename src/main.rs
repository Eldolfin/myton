use std::io;
mod token;

fn main() -> io::Result<()> {
    loop{
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
       
        let mut lexer = token::Lexer::new(input);
        
        let tokens = lexer.tokenize();

        println!("{:?}", tokens);
        
    }
    Ok(())
}
