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
    pub fn alterar_senha(&self){
        utilitarios::limpar_tela();
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
        
        println!("aqui esta uma lista com todas as suas senhas: ");
        for (i,(url_site, usuario_site, nonce, senha_criptografada)) in lista_senhas.iter().enumerate() {
            let chave: [u8; 32] = chave_usuario
                .as_slice()
                .try_into()
                .expect("Chave de criptografia inválida");
            let senha = decrypt(&chave, &nonce, &senha_criptografada);

            println!("Site numero {}",i);
            println!("Nome do Site: {}", url_site);
            println!("Nome de Usuário do Site: {}", usuario_site);
            println!(
                "Senha: {}",
                String::from_utf8(senha).expect("Senha inválida")
            );
            println!("=======================================================================");
            println!();
        }

        println!("Selecione o número do site que deseja alterar a senha: ");
        let mut escolha_site = String::new();
        io::stdin().read_line(&mut escolha_site).expect("Erro ao ler entrada");
        let escolha_site: usize = escolha_site.trim().parse().expect("Entrada inválida");

        if escolha_site >= lista_senhas.len(){
            println!("Opção invalida!");
            return;
        }

        println!("Digite a nova senha: ");
        let mut nova_senha = String::new();
        io::stdin().read_line(&mut nova_senha).expect("Erro ao ler a nova senha");
        let nova_senha = nova_senha.trim();

        println!("Digite a nova senha novamente: ");
        let mut nova_senha2 = String::new();
        io::stdin().read_line(&mut nova_senha2).expect("Erro ao ler a nova senha");
        let nova_senha2 = nova_senha2.trim();

        if nova_senha != nova_senha2{
            println!("As senhas nao coincidem.");
            return;
        }

        let chave:[u8; 32] = chave_usuario.as_slice().try_into().expect("Chave de criptografia invalida.");
        let(nonce, senha_criptografada) = encrypt(&chave, &nova_senha.as_bytes());

        let (url_site, usuario_site, _, _) = &lista_senhas[escolha_site];

        conn.exec_drop(
            "UPDATE senhas SET senha_site = :nova_senha , nonce = :nonce WHERE url_site = :url AND usuario_site = :usuario AND chave_criptografia_usuario = :chave",
            params! {
                "nova_senha" => senha_criptografada,
                "nonce" => nonce,
                "url" => url_site,
                "usuario" => usuario_site,
                "chave" => chave_usuario,
            }
        ).expect("Erro ao atualizar a senha no banco de dados");
    
        println!("Senha alterada com sucesso!");
    }
    pub fn excluir_senha(&self){
        utilitarios::limpar_tela();
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
        
        println!("aqui esta uma lista com todas as suas senhas: ");
        for (i,(url_site, usuario_site, nonce, senha_criptografada)) in lista_senhas.iter().enumerate() {
            let chave: [u8; 32] = chave_usuario
                .as_slice()
                .try_into()
                .expect("Chave de criptografia inválida");
            let senha = decrypt(&chave, &nonce, &senha_criptografada);

            println!("Site numero {}",i);
            println!("Nome do Site: {}", url_site);
            println!("Nome de Usuário do Site: {}", usuario_site);
            println!(
                "Senha: {}",
                String::from_utf8(senha).expect("Senha inválida")
            );
            println!("=======================================================================");
            println!();
        }
        println!("Qual das senhas deseja excluir: ");
        let mut escolha =String::new();
        io::stdin().read_line(&mut escolha).expect("Nao foi possivel ler a entrada");
        let escolha :usize = escolha.trim().parse().expect("Erro ao escolher a entrada");

        if escolha >= lista_senhas.len(){
            println!("entrada invalida");
            return;
        }

        let chave:[u8; 32] = chave_usuario.as_slice().try_into().expect("Chave de criptografia invalida.");
        let(url_site, usuario_site, nonce, senha_criptografada) = &lista_senhas[escolha];
        println!("Certeza que deseja excluir as seguinte conta:");
        println!("Site/url: {:?}", &url_site);
        println!("Usuario: {:?}", &usuario_site);
        println!("senha: {:?}", String::from_utf8(decrypt(&chave, &nonce, &senha_criptografada)).expect("Senha invalida"));
        println!("Digite 'S' para sim e 'N' para não");
        let mut certeza = String::new();
        io::stdin().read_line(&mut certeza).expect("Nao foi possivel ler a entrada");
        let certeza:char = certeza.trim().parse().expect("Erro na conversao");
        if certeza !='S' && certeza !='s' && certeza != 'N' && certeza != 'n'{
            println!("Opção invalida");
            return;
        }
        if certeza == 'S' || certeza == 's'{
            conn.exec_drop("DELETE FROM senhas WHERE chave_criptografia_usuario = :chave_usuario AND usuario_site = :usuario_site AND url_site = :url_site", 
            params!{
                "chave_usuario" =>chave_usuario,
                "usuario_site" =>usuario_site,
                "url_site" =>url_site,
            }).expect("Erro ao excluir senha");
        }
    }
}
