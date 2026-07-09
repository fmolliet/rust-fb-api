# fbrustapi

Microserviço REST API de alta performance construído em **Rust** utilizando **Axum** e **SQLx** (com backend **PostgreSQL**).

Este projeto é uma reescrita moderna e otimizada de uma implementação Java anterior (utilizando RESTEasy e Quarkus) para gerenciar processos de registro e rastreamento de aniversários baseados em IDs de snowflake do Discord.

---

## 🚀 Tecnologias e Stack

O microserviço é construído sobre as seguintes fundações:

| Camada | Tecnologia | Descrição |
|---|---|---|
| **Web Framework** | [Axum](https://github.com/tokio-rs/axum) | Framework HTTP assíncrono e ergonômico construído sobre `hyper` |
| **Banco de Dados** | [SQLx](https://github.com/launchbadge/sqlx) | Driver SQL assíncrono com validação de queries em tempo de compilação |
| **Runtime Assíncrono** | [Tokio](https://github.com/tokio-rs/tokio) | Runtime assíncrono padrão da indústria para Rust |
| **Validação** | `validator` + `serde` | Serialização rápida e validação estrita de DTOs |
| **Logs & Tracing** | `tracing` + `tracing-subscriber` | Logging estruturado e observabilidade diagnóstica |
| **Identificadores** | `uuid v7` | UUIDs baseados em tempo (Unix epoch) para melhor ordenação no DB |
| **Datas** | `chrono` | Manipulação de fusos horários configurado para GMT-3 |

---

## 🛠️ Requisitos de Ambiente

- [Rust](https://www.rust-lang.org/) (incluindo `cargo`)
- [Docker](https://www.docker.com/) ou [Colima](https://github.com/abiosoft/colima) (para rodar o banco PostgreSQL localmente)
- `curl` (usado nos testes locais)

---

## 📦 Inicialização e Testes Locais Simplicados

Para simplificar o desenvolvimento e a execução dos testes de integração locais, foi criado o script utilitário `run_local.sh`. Ele gerencia automaticamente o ciclo de vida do banco de dados PostgreSQL via Docker/Colima e executa o servidor local.

### Comandos Disponíveis

```bash
# Exibe a ajuda com todas as opções
./run_local.sh help

# Modo Desenvolvimento (Inicia o DB, aplica migrações e executa cargo run em foreground)
./run_local.sh dev

# Modo Teste de Integração (Inicia o DB, aplica migrações, compila e roda a API em background, valida com curl e encerra)
./run_local.sh test

# Limpeza (Para e remove o container Docker do banco de dados local)
./run_local.sh clean

# Reset (Recria o schema público do banco do zero e reaplica todas as migrações)
./run_local.sh reset
```

*Nota: O script detecta automaticamente se o Docker daemon não está rodando e tenta iniciar o Colima se estiver instalado no macOS.*

---

## 🗄️ Esquema do Banco de Dados (PostgreSQL)

O banco de dados contém duas tabelas principais configuradas em `migrations/`:

### 1. `registrations`
Armazena os processos de registro/aprovação de novos usuários.
- Protegido por um **Partial Unique Index** (`uq_open_registration`) que garante que cada solicitante (`applicant_id`) possa ter apenas **um** registro aberto (`finished = FALSE`) por vez.

```sql
CREATE TABLE registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    applicant_id TEXT NOT NULL,
    curator_id TEXT,
    birthday TEXT,
    specie TEXT,
    source TEXT,
    message_id TEXT,
    finished BOOLEAN DEFAULT FALSE,
    approved BOOLEAN,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 2. `birthdays`
Gerencia datas de aniversário associadas a um ID snowflake do Discord.
- O campo `snowflake` funciona como chave primária.

```sql
CREATE TABLE birthdays (
    snowflake TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    day INTEGER NOT NULL CHECK (day BETWEEN 1 AND 31),
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 🌐 Endpoints Planejados da API

### Módulo de Registros (`/registration`)

| Método | Rota | Descrição |
|---|---|---|
| `POST` | `/registration` | Cria uma solicitação de registro (bloqueia se já houver registro aberto para o `applicant_id`) |
| `GET` | `/registration/open/{applicant_id}` | Obtém a solicitação de registro ativa de um solicitante |
| `GET` | `/registration/{id}` | Busca um registro específico por ID |
| `PATCH` | `/registration/{id}/finish` | Finaliza e aprova/rejeita uma solicitação de registro |
| `DELETE` | `/registration/{id}` | Remove um registro específico do sistema |

### Módulo de Aniversários (`/birthday`)

| Método | Rota | Descrição |
|---|---|---|
| `GET` | `/birthday` | Lista aniversariantes de forma paginada (parâmetros `page` e `size`) |
| `GET` | `/birthday/today` | Lista aniversariantes do dia atual (calculado sob o fuso GMT-3) |
| `GET` | `/birthday/month` | Lista todos os aniversariantes do mês corrente (GMT-3) |
| `GET` | `/birthday/{snowflake}` | Obtém o aniversário cadastrado de um snowflake específico |
| `POST` | `/birthday` | Cadastra o aniversário de um snowflake (retorna 400 se já existir) |
| `PATCH` | `/birthday/{snowflake}` | Atualiza o aniversário de um snowflake existente |
| `DELETE` | `/birthday/{snowflake}` | Remove o aniversário cadastrado de um snowflake |

---

## ✨ Melhorias em relação à Implementação Original (Java)

1. **Validação Robusta no Roteamento**: URLs com UUIDs inválidos retornam `400 Bad Request` imediatamente na camada de roteamento, antes de atingirem o banco.
2. **Histórico Temporal Completo**: Presença automática de `created_at` e `updated_at` (com timezone UTC/GMT-3) para melhor auditoria.
3. **API PATCH Semântica**: `/registration/{id}/finish` expõe claramente a ação pretendida no recurso, em vez de enviar payloads de alteração genéricos.
4. **Resolução Clara de Conflitos**: A rota `/registration/open/{applicant_id}` é tratada de forma explícita no roteador Axum, eliminando conflitos de interpretação com rotas de ID UUID.
5. **Retornos Semânticos Estritos**: Retorno correto de `404 Not Found` quando um registro não existe, em vez de retornar payloads vazios/nulos.
6. **Concorrência Segura**: O índice único parcial no banco de dados assegura a integridade de registros abertos mesmo em condições de alta concorrência paralela.
7. **Logging Estruturado**: Tracing assíncrono estruturado no lugar de logs de texto livre.
8. **Erros Fortemente Tipados**: Uso de `AppError` integrado com as respostas HTTP do Axum para códigos de status precisos.
