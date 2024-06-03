#![allow(unused_variables)]
#![allow(unreachable_code)]

use crate::criptografia::*;
use crate::utilitarios;
use mysql::prelude::*;
use mysql::*;
use rand::rngs::OsRng;
use rand::RngCore;
use rpassword::prompt_password;
use std::io::{self, Write};

pub struct BancoDadosConn {
    pool: Pool,
    chave_usuario_logado: Option<Vec<u8>>,
}
impl BancoDadosConn {
    pub fn new(url: &str) -> Self {
        let pool = Pool::new(url).expect("Erro ao criar pool de conexão");
        BancoDadosConn {
            pool,
            chave_usuario_logado: None,
        }
    }

    pub fn gerar_chave_aleatoria() -> [u8; 32] {
        let mut chave = [0u8; 32];
        OsRng.fill_bytes(&mut chave);
        chave
    }

    pub fn criar_usuario(&self) -> bool {
        let mut conn = self.pool.get_conn().expect("Erro ao obter a conexão");

        //gera a chave aleatoria.
        let chave = BancoDadosConn::gerar_chave_aleatoria();
        //solicita o email
        print!("Digite o email:  ");
        io::stdout().flush().unwrap();
        let mut email = String::new();
        io::stdin()
            .read_line(&mut email)
            .expect("Falha ao ler o email");
        let email = email.trim();
        //solicita o nome de usuario
        print!("Digite o nome de usuario:  ");
        io::stdout().flush().unwrap();
        let mut nome_usuario = String::new();
        io::stdin()
            .read_line(&mut nome_usuario)
            .expect("Falha ao ler o nome de usuario");
        let nome_usuario = nome_usuario.trim();
        //solicita a senha principal
        let senha = prompt_password("Digite a senha: ").expect("Falha ao ler a senha");
        let senha = senha.trim();
        //solicita a senha principal
        let senha_novamente =
            prompt_password("Digite novamente a senha: ").expect("Falha ao ler a senha");
        let senha_novamente = senha_novamente.trim();
        //verifica se todas as entradas estao no tamanho correto
        if utilitarios::validar_email(&email)
            && senha == senha_novamente
            && utilitarios::validar_senha(&senha)
            && utilitarios::validar_usuario(&nome_usuario)
        {
            conn.exec_drop(
            r"INSERT INTO usuarios (chave_criptografia, usuario, email, senha) VALUES (:chave, :usuario, :email, :senha)",
            params! {
                "chave" => &chave[..],
                "usuario" => &nome_usuario[..],
                "email" => &email[..],
                "senha" => &senha[..],
            }
        ).expect("Erro ao inserir dados no banco de dados");
            true
        } else {
            false
        }
    }

    pub fn consulta_bd(&self) {
        let mut conn = self.pool.get_conn().expect("Erro ao obter a conexão");

        let lista_linhas: Vec<(Vec<u8>, String, String, String)> = conn
            .query("SELECT chave_criptografia, usuario, email, senha FROM usuarios")
            .expect("Erro ao consultar dados do banco de dados");

        // Exibe todos os dados armazenados
        for linha in lista_linhas {
            println!("Chave_criptografia:  {:?}", linha.0);
            println!("Usuario: {:?}", linha.1);
            println!("Email: {:?}", linha.2);
            println!("Senha: {:?}", linha.3);
            println!();
        }
    }

    pub fn fazer_login(&mut self) -> bool {
        let mut conn = self.pool.get_conn().expect("Erro ao obter conexão do pool");

        // solicita o usuário
        print!("Digite o seu usuario:  ");
        io::stdout().flush().unwrap();
        let mut nome_usuario = String::new();
        io::stdin()
            .read_line(&mut nome_usuario)
            .expect("Falha ao ler o seu nome");
        let nome_usuario = nome_usuario.trim();

        // solicita a senha principal
        let senha = prompt_password("Digite a senha: ").expect("Falha ao ler a senha");
        let senha = senha.trim();

        let lista_linhas: Vec<(Vec<u8>, String, String)> = conn.exec(
            "SELECT chave_criptografia, usuario, senha FROM usuarios WHERE usuario = :usuario AND senha = :senha",
            params! {
                "usuario" => nome_usuario,
                "senha" => senha,
            }
        ).expect("Erro ao consultar dados do banco de dados");

        if let Some((chave, _, _)) = lista_linhas.into_iter().next() {
            self.chave_usuario_logado = Some(chave);
            true
        } else {
            false
        }
    }

    pub fn inserir_senha(&self) {
        if self.chave_usuario_logado.is_none() {
            println!("Você precisa fazer login primeiro.");
            return;
        }

        let chave_usuario = self.chave_usuario_logado.as_ref().unwrap();
        let mut conn = self.pool.get_conn().expect("Erro ao obter conexão do pool");

        // solicita o nome do site
        print!("Digite o nome/url do site: ");
        io::stdout().flush().unwrap();
        let mut url_site = String::new();
        io::stdin()
            .read_line(&mut url_site)
            .expect("Falha ao ler o nome do site");
        let url_site = url_site.trim();

        // solicita o nome de usuário do site
        print!("Digite o nome de usuário do site: ");
        io::stdout().flush().unwrap();
        let mut usuario_site = String::new();
        io::stdin()
            .read_line(&mut usuario_site)
            .expect("Falha ao ler o nome de usuário do site");
        let usuario_site = usuario_site.trim();

        // solicita a senha
        print!("Digite a senha: ");
        io::stdout().flush().unwrap();
        let mut senha = String::new();
        io::stdin()
            .read_line(&mut senha)
            .expect("Falha ao ler a senha");
        let senha = senha.trim();

        // criptografa a senha
        let chave: [u8; 32] = chave_usuario
            .as_slice()
            .try_into()
            .expect("Chave de criptografia inválida");
        let (nonce, senha_criptografada) = encrypt(&chave, senha.as_bytes());

        // insere os dados na tabela senhas
        conn.exec_drop(
            r"INSERT INTO senhas (chave_criptografia_usuario, url_site, usuario_site, nonce, senha_site) VALUES (:chave, :url_site, :usuario_site, :nonce, :senha)",
            params! {
                "chave" => chave_usuario,
                "url_site" => url_site,
                "usuario_site" => usuario_site,
                "nonce" => nonce,
                "senha" => senha_criptografada,
            }
        ).expect("Erro ao inserir dados na tabela senhas");

        println!("Senha inserida com sucesso.");
    }

    pub fn consultar_senhas(&self) {
        if self.chave_usuario_logado.is_none() {
            println!("Você precisa fazer login primeiro.");
            return;
        }

        let chave_usuario = self.chave_usuario_logado.as_ref().unwrap();
        let mut conn = self.pool.get_conn().expect("Erro ao obter conexão do pool");

        let lista_senhas: Vec<(String, String, Vec<u8>, Vec<u8>)> = conn.exec(
            "SELECT url_site, usuario_site, nonce, senha_site FROM senhas WHERE chave_criptografia_usuario = :chave",
            params! {
                "chave" => chave_usuario,
            }
        ).expect("Erro ao consultar dados do banco de dados");

        for (url_site, usuario_site, nonce, senha_criptografada) in lista_senhas {
            let chave: [u8; 32] = chave_usuario
                .as_slice()
                .try_into()
                .expect("Chave de criptografia inválida");
            let senha = decrypt(&chave, &nonce, &senha_criptografada);

            println!("Nome do Site: {}", url_site);
            println!("Nome de Usuário do Site: {}", usuario_site);
            println!(
                "Senha: {}",
                String::from_utf8(senha).expect("Senha inválida")
            );
            println!();
        }
    }
}
