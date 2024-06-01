#![allow(unused_variables)]
#![allow(unreachable_code)]

use mysql::*;
use mysql::prelude::*;
use rand::RngCore;
use rand::rngs::OsRng;
use std::io::{self, Write};
use crate::utilitarios;

/*pub fn conectar_banco_dados(){
    // Configura a URL de conexão com o banco de dados
    let url = "mysql://myadmin:357159Vinic@meuserverdemo-mysql.mysql.database.azure.com:3306/projeto_rust";
    // Cria uma pool de conexões
    let pool = Pool::new(url).expect("Erro ao criar pool de conexões");
    //obtém uma conexão
    let mut conn = pool.get_conn().expect("Erro ao obter conexão do pool");
}*/

pub fn gerar_chave_aleatoria() -> [u8; 32] {
    let mut chave = [0u8; 32];
    OsRng.fill_bytes(&mut chave);
    chave
}

pub fn criar_usuario() -> bool{
    // Configura a URL de conexão com o banco de dados
    let url = "mysql://myadmin:357159Vinic@meuserverdemo-mysql.mysql.database.azure.com:3306/projeto_rust";
    // Cria uma pool de conexões
    let pool = Pool::new(url).expect("Erro ao criar pool de conexões");
    //obtém uma conexão
    let mut conn = pool.get_conn().expect("Erro ao obter conexão do pool");

    //gera a chave aleatoria.
    let chave = gerar_chave_aleatoria();
    //solicita o email
    print!("Digite o email:  ");
    io::stdout().flush().unwrap();
    let mut email = String::new();
    io::stdin().read_line(&mut email).expect("Falha ao ler o email");
    let email = email.trim();
    //solicita o nome de usuario
    print!("Digite o nome de usuario:  ");
    io::stdout().flush().unwrap();
    let mut nome_usuario = String::new();
    io::stdin().read_line(&mut nome_usuario).expect("Falha ao ler o nome de usuario");
    let nome_usuario = nome_usuario.trim();
    //solicita o nome
    print!("Digite o seu nome:  ");
    io::stdout().flush().unwrap();
    let mut nome_pessoa = String::new();
    io::stdin().read_line(&mut nome_pessoa).expect("Falha ao ler o seu nome");
    let nome_pessoa = nome_pessoa.trim();
    //solicita a senha principal
    print!("Digite a senha:  ");
    io::stdout().flush().unwrap();
    let mut senha = String::new();
    io::stdin().read_line(&mut senha).expect("Falha ao ler a senha");
    let senha = senha.trim();
    //verifica se todas as entradas estao no tamanho correto
    let mut resultado:bool = if utilitarios::validar_email(&email) && utilitarios::validar_nome(&nome_pessoa) && utilitarios::validar_senha(&senha) && utilitarios::validar_usuario(&nome_usuario){
        conn.exec_drop(
            r"INSERT INTO usuarios (chave_criptografia, usuario, email, senha, nome) VALUES (:chave, :usuario, :email, :senha, :nome)",
            params! {
                "chave" => &chave[..],
                "usuario" => &nome_usuario[..],
                "email" => &email[..],
                "senha" => &senha[..],
                "nome" => &nome_pessoa[..],
            }
        ).expect("Erro ao inserir dados no banco de dados");
        return true
    } else{
        return false
    };
    resultado
}

pub fn consulta_bd(){
        // Configura a URL de conexão com o banco de dados
        let url = "mysql://myadmin:357159Vinic@meuserverdemo-mysql.mysql.database.azure.com:3306/projeto_rust";
        // Cria uma pool de conexões
        let pool = Pool::new(url).expect("Erro ao criar pool de conexões");
        //obtém uma conexão
        let mut conn = pool.get_conn().expect("Erro ao obter conexão do pool");

    let lista_linhas: Vec<(Vec<u8>, String, String, String, String)> = conn.query(
        "SELECT chave_criptografia, usuario, email, senha, nome FROM usuarios"
    ).expect("Erro ao consultar dados do banco de dados");

    // Exibe todos os dados armazenados
    for linha in lista_linhas {
        println!("Chave_criptografia:  {:?}", linha.0);
        println!("Usuario: {:?}", linha.1);
        println!("Email: {:?}", linha.2);
        println!("Senha: {:?}", linha.3);
        println!("Nome: {}", linha.4);
        println!();
    }
}
pub fn fazer_login() -> (bool, String, String){
    // Configura a URL de conexão com o banco de dados
    let url = "mysql://myadmin:357159Vinic@meuserverdemo-mysql.mysql.database.azure.com:3306/projeto_rust";
    // Cria uma pool de conexões
    let pool = Pool::new(url).expect("Erro ao criar pool de conexões");
    //obtém uma conexão
    let mut conn = pool.get_conn().expect("Erro ao obter conexão do pool");

    //solicita o usuario
    print!("Digite o seu usuario:  ");
    io::stdout().flush().unwrap();
    let mut nome_usuario = String::new();
    io::stdin().read_line(&mut nome_usuario).expect("Falha ao ler o seu nome");
    let nome_usuario = nome_usuario.trim();
    //solicita a senha principal
    print!("Digite a senha:  ");
    io::stdout().flush().unwrap();
    let mut senha = String::new();
    io::stdin().read_line(&mut senha).expect("Falha ao ler a senha");
    let senha = senha.trim();

    let lista_linhas: Vec<(Vec<u8>, String, String)> = conn.query(
        "SELECT chave_criptografia, usuario, senha FROM usuarios"
    ).expect("Erro ao consultar dados do banco de dados");

    for linha in lista_linhas{
        if linha.1 == nome_usuario as String && lista.2 == senha as String{
            return(true, nome_usuario, senha);
        }else{
            return(false, nome_usuario, senha);
        }
    }

}