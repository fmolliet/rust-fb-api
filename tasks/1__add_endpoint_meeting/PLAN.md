# Plano de Migração: Meeting Service — Monolito

## 1. Visão Geral do Serviço

| Aspecto | Detalhe |
|---------|---------|
| **Nome** | meeting-service |
| **Tech Atual** | Java 17, Spring Boot 3.1.2, JPA/Hibernate, PostgreSQL, Flyway |
| **GAV** | `io.winty.struct:meeting-service:1.0.0-SNAPSHOT` |
| **Porta** | 8080 (configurável via `PORT`) |
| **Database** | PostgreSQL (já é Postgres) |
| **Migrations** | Flyway (4 scripts: V1-V3 legacy `medicos`, V4 `meeting`) |
| **Camada Service** | Não existe — Controller injeta Repository diretamente |
| **Testes** | Apenas `contextLoads()` — sem testes de negócio |

---

## 2. Rotas da API (Base: `/meeting`)

### 2.1 `GET /meeting` — Listar reuniões (paginado)

| Campo | Valor |
|-------|-------|
| **Método** | `GET` |
| **Path** | `/meeting` |
| **Query Params** | `page` (int), `size` (int, default 1000), `sort` (String) |
| **Response** | `Page<MeetingDTO>` — apenas reuniões `active = true` |
| **Status** | 200 OK |

**Response Body (200):**
```json
{
  "content": [
    {
      "id": 1,
      "name": "Reunião de Planejamento",
      "snowflake": "abc-123-def",
      "state": "SP"
    }
  ],
  "pageable": { ... },
  "totalElements": 50,
  "totalPages": 1,
  "number": 0,
  "size": 1000
}
```

**Erros:**
- `500 Internal Server Error` — se falhar na query

---

### 2.2 `GET /meeting/{id}` — Buscar por snowflake

| Campo | Valor |
|-------|-------|
| **Método** | `GET` |
| **Path** | `/meeting/{id}` |
| **Path Param** | `id` (String) — na verdade é o campo `snowflake`, não o `id` numérico |
| **Response** | `MeetingDTO` ou `null` |
| **Status** | 200 OK (ou `null` body se não encontrado) |

**Response Body (200):**
```json
{
  "id": 1,
  "name": "Reunião de Planejamento",
  "snowflake": "abc-123-def",
  "state": "SP"
}
```

**Response Body (not found):**
```
null
```

**Erros:**
- Qualquer exceção é capturada, logada como WARNING, retorna `200 null`
- **Não há 404** — bug conhecido

---

### 2.3 `GET /meeting/state/{state}` — Listar por estado (paginado)

| Campo | Valor |
|-------|-------|
| **Método** | `GET` |
| **Path** | `/meeting/state/{state}` |
| **Path Param** | `state` (String, 2 chars — código do estado) |
| **Query Params** | `page`, `size` (default 1000), `sort` |
| **Response** | `Page<MeetingDTO>` — apenas reuniões ativas do estado |
| **Status** | 200 OK |

**Response Body (200):**
```json
{
  "content": [ ... ],
  "totalElements": 5,
  "totalPages": 1,
  "number": 0,
  "size": 1000
}
```

**Erros:**
- Qualquer exceção retorna `200 null`

---

### 2.4 `POST /meeting` — Criar reunião

| Campo | Valor |
|-------|-------|
| **Método** | `POST` |
| **Path** | `/meeting` |
| **Content-Type** | `application/json` |
| **Request Body** | `CreateMeetingDTO` |
| **Response** | 200 OK, sem body |
| **Transaction** | `@Transactional` |

**Request Body:**
```json
{
  "name": "Reunião de Planejamento",
  "snowflake": "abc-123-def",
  "state": "SP"
}
```

| Campo | Obrigatório | Tipo | Validação |
|-------|-------------|------|-----------|
| `name` | Não | `String` | livre |
| `snowflake` | Sim | `String` | `@NotBlank` |
| `state` | Sim | `String` | `@NotBlank` |

**Erros:**
- `400 Bad Request` — validação falha (`@NotBlank`)
- `409 Conflict` — `snowflake` duplicado (unique constraint no DB)

---

### 2.5 `PUT /meeting` — Atualizar reunião

| Campo | Valor |
|-------|-------|
| **Método** | `PUT` |
| **Path** | `/meeting` |
| **Content-Type** | `application/json` |
| **Request Body** | `UpdateMeetingDTO` |
| **Response** | 200 OK, sem body |
| **Transaction** | `@Transactional` |

**Request Body:**
```json
{
  "name": "Reunião Atualizada",
  "snowflake": "abc-123-def",
  "state": "RJ"
}
```

| Campo | Obrigatório | Tipo | Validação |
|-------|-------------|------|-----------|
| `name` | Não | `String` | nullable (partial update) |
| `snowflake` | Sim | `String` | `@NotNull @NotBlank` |
| `state` | Sim | `String` | `@NotNull @NotBlank @Size(min=2)` |

**Comportamento:**
- Partial update: apenas campos não-null são alterados
- Sempre reativar a reunião (`active = true`) ao atualizar
- Se `snowflake` não existe → **NPE** (bug — null é retornado por `findBySnowflake`)

**Erros:**
- `400 Bad Request` — validação falha
- `500 Internal Server Error` — snowflake não encontrado (NPE)

---

### 2.6 `DELETE /meeting/{id}` — Desativar reunião (soft delete)

| Campo | Valor |
|-------|-------|
| **Método** | `DELETE` |
| **Path** | `/meeting/{id}` |
| **Path Param** | `id` (Long — ID numérico, NÃO snowflake) |
| **Response** | 200 OK, sem body |
| **Transaction** | `@Transactional` |

**Erros:**
- `404 Not Found` — ID não existe (`EntityNotFoundException` do Spring Data)
- Soft delete: define `active = false`, não remove do DB

---

## 3. Contratos de Dados

### 3.1 `MeetingDTO` (response)

```json
{
  "id":        "Long — ID auto-incremento",
  "name":      "String — até 200 chars",
  "snowflake": "String — único, até 50 chars",
  "state":     "String — código do estado, até 2 chars"
}
```

### 3.2 `CreateMeetingDTO` (request creation)

```json
{
  "name":      "String — opcional",
  "snowflake": "String — obrigatório, não vazio",
  "state":     "String — obrigatório, não vazio"
}
```

### 3.3 `UpdateMeetingDTO` (request update)

```json
{
  "name":      "String — opcional (partial update)",
  "snowflake": "String — obrigatório, não vazio, não null",
  "state":     "String — obrigatório, não vazio, não null, min 2 chars"
}
```

---

## 4. Modelo de Dados Atual

### Tabela `meeting`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | `bigserial` (bigint auto-increment) | `PRIMARY KEY` |
| `name` | `varchar(200)` | `NOT NULL` |
| `snowflake` | `varchar(50)` | `NOT NULL UNIQUE` |
| `state` | `varchar(2)` | `NOT NULL` |
| `active` | `boolean` | `NOT NULL DEFAULT true` |

### Índices

| Nome | Coluna | Único? |
|------|--------|--------|
| `meeting_pkey` | `id` | Sim (PK) |
| `meeting_snowflake_key` | `snowflake` | Sim |

---

## 5. Migração para Monolito

### 5.1 O que precisa ser feito

1. **Copiar as rotas** (6 endpoints) para o controlador do monolito
2. **Copiar os DTOs** (3 records: `MeetingDTO`, `CreateMeetingDTO`, `UpdateMeetingDTO`)
3. **Migrar o entity** `Meeting` para o esquema do monolito
4. **Manter os mesmos paths** (`/meeting/*`), status codes e formatos de resposta
5. **Reimplementar a camada de serviço** — o serviço atual não tem camada de service
6. **Implementar exception handling** — o serviço atual retorna `null` em vez de 404 e tem NPE no update

### 5.2 Quebras de compatibilidade a corrigir

| Problema | Impacto | Ação |
|----------|---------|------|
| `GET /meeting/{id}` retorna `null` (200) em vez de 404 | Clientes podem falhar se esperarem objeto | Lançar `EntityNotFoundException` ou retornar `ResponseEntity.notFound()` |
| `PUT /meeting` NPE se snowflake não existe | 500 em vez de 404 | Validar existence antes de `update()` |
| `DELETE /meeting/{id}` usa ID numérico, mas `GET` usa snowflake no mesmo path | Design inconsistente | Manter como-is para compatibilidade, mas documentar |
| Page size default = 1000 | Performance ruim | Redefinir para valor menor (ex: 25-50) no monolito |
| `state/{state}` retorna `null` (200) em vez de lista vazia ou 404 | Inconsistência | Retornar `Page.empty()` |
| Sem `@ControllerAdvice` | Erros retornam formato default Spring, sem padrão | Implementar global error handler no monolito |

### 5.3 Decisões de arquitetura

| Decisão | Recomendação |
|---------|-------------|
| Manter `/meeting` como path? | **Sim** — compatibilidade total |
| Manter soft delete? | **Sim** — campo `active` já existe nos dados |
| Manter snowflake como chave de lookup? | **Sim** — já é único e usado pelos clientes |
| Remover migrations `medicos` (V1-V3)? | **Sim** — são legacy e não usadas |

---

## 6. Migração do Banco de Dados

### 6.1 Contexto

O banco **já é PostgreSQL**. A migração é de **estrutura de table**, não de banco.

### 6.2 Tabela existente

A tabela `meeting` já tem o esquema correto. Não é necessário migrar dados entre tecnologias.

### 6.3 Se precisar recriar a tabela no novo banco

```sql
CREATE TABLE IF NOT EXISTS meeting (
    id      BIGSERIAL PRIMARY KEY,
    name    VARCHAR(200) NOT NULL,
    snowflake VARCHAR(50) NOT NULL UNIQUE,
    state   VARCHAR(2)   NOT NULL,
    active  BOOLEAN      NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_meeting_state_active ON meeting(state, active);
```

### 6.4 Flyway migrations para o monolito

```
V1__create_meeting_table.sql
V2__add_index_meeting_state_active.sql
```

Remover V1-V3 (medicos) pois são do serviço antigo.

### 6.5 Estratégias de migração de dados

#### Estratégia recomendada: Blue-Green com dual-write

1. **Fase 1 — Parallel run**
   - Novo monolito lê do mesmo banco Postgres
   - Escreve nas mesmas tabelas (se ambos escrevem, usar lock ou evitar overlap)
   - Ou: novo monolito lê do banco existente, escreve em paralelo
   - Validar dados idênticos

2. **Fase 2 — Cutover**
   - Desligar serviço Java antigo
   - Monolito assume todas as escritas
   - Validar sem erros

3. **Fase 3 — Cleanup**
   - Remover código legado
   - Remover migrations `medicos`
   - Ajustar page size default

### 6.6 Mapeamento de tipos (Spring Boot → Monolito)

| Tipo Atual | Tipo no DB | Equivalente no Monolito |
|------------|-----------|------------------------|
| `Long` (id) | `BIGSERIAL` | `BigInt` / `long` |
| `String` (name) | `VARCHAR(200)` | `String` / `str` |
| `String` (snowflake) | `VARCHAR(50)` | `String` / `str` |
| `String` (state) | `VARCHAR(2)` | `String` / `str` |
| `Boolean` (active) | `BOOLEAN` | `bool` / `Boolean` |

### 6.7 Índices necessários

| Índice | Coluna(s) | Razão |
|--------|-----------|-------|
| `meeting_pkey` | `id` | PK — obrigatório |
| `meeting_snowflake_key` | `snowflake` | `UNIQUE` — obrigatório |
| `idx_meeting_state_active` | `state, active` | Otimiza `findAllByActiveTrueAndState()` |
| `idx_meeting_active` | `active` | Otimiza `findAllByActiveTrue()` |

---

## 7. Checklist de Migração

### Análise ✅
- [x] Mapear todas as rotas
- [x] Documentar contratos de dados (DTOs)
- [x] Identificar comportamento de erros
- [x] Mapear modelo de dados

### Implementação
- [ ] Criar controlador no monolito com as 6 rotas
- [ ] Copiar DTOs com mesmas validações (`@NotBlank`, `@NotNull`, `@Size`)
- [ ] Implementar entity/model `Meeting` com campos `id`, `name`, `snowflake`, `state`, `active`
- [ ] Implementar camada de service (resolvendo o problema de não ter no serviço original)
- [ ] Implementar repositório com os 3 query methods
- [ ] Implementar exception handler global (`@ControllerAdvice`)
- [ ] Corrigir `GET /meeting/{id}` para retornar 404 em vez de `null`
- [ ] Corrigir `PUT /meeting` para retornar 404 se snowflake não existe
- [ ] Corrigir `GET /meeting/state/{state}` para retornar `Page.empty()` em vez de `null`
- [ ] Configurar page size default menor (25-50)
- [ ] Criar Flyway migrations V1, V2 para table `meeting`
- [ ] Migrar dados existentes do banco (se necessário)

### Testes
- [ ] Testes unitários para cada endpoint (CRUD completo)
- [ ] Testes de validação (campos obrigatórios, `@NotBlank`, `@Size`)
- [ ] Testes de erro (404, 400, 409 unique constraint)
- [ ] Testes de paginação
- [ ] Testes de soft delete
- [ ] Testes de partial update (PUT com campos null)

### Deploy
- [ ] Deploy canário do monolito com rotas meeting
- [ ] Validar compatibilidade com clientes existentes
- [ ] Desligar serviço Java antigo
- [ ] Remover código legado do repositório
- [ ] Atualizar documentação
