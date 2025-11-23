# RDS CLI

[![CI](https://github.com/junyeong-ai/rds-cli/workflows/CI/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Lint](https://github.com/junyeong-ai/rds-cli/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Rust](https://img.shields.io/badge/rust-1.91.1%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/junyeong-ai/rds-cli/releases)

> **🌐 한국어** | **[English](README.en.md)**

---

> **⚡ PostgreSQL/MySQL을 위한 빠르고 안전한 Database CLI**
>
> - 🚀 **초고속** (Rust 기반, <5ms 스키마 조회)
> - 🔒 **프로덕션 안전** (자동 LIMIT, 읽기 전용)
> - 📝 **팀 협업** (Git 버전 관리 Named Queries)
> - 🔍 **스마트 검색** (퍼지 매칭, 자동 완성)

---

## 왜 RDS CLI인가?

기존 DB 클라이언트는 **느리고**, **위험하고**, **팀 협업이 어렵습니다**.

| 기존 방식 | RDS CLI |
|---------|---------|
| 🐌 매번 스키마 조회 (수백ms) | ⚡ 캐싱으로 <5ms 조회 |
| ❌ 실수로 전체 테이블 조회 | ✅ 자동 LIMIT 적용 |
| 🔓 프로덕션에서 DELETE 가능 | 🔒 읽기 전용 강제 |
| 📋 복잡한 쿼리를 매번 복붙 | 📝 팀 공유 Named Queries |
| 🤷 오타 시 "테이블 없음" | 🔍 퍼지 검색으로 제안 |

---

## ⚡ 빠른 시작

```bash
# 1. 설치 (1줄)
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash

# 2. 설정 (1분)
rds-cli config init
rds-cli config edit  # DB 정보 입력

# 3. 스키마 캐싱
export DB_PASSWORD_LOCAL="your-password"
rds-cli refresh

# 4. 사용 시작!
rds-cli schema find user
rds-cli query "SELECT * FROM users"
```

---

## 🎯 주요 기능

### 1. 번개같이 빠른 스키마 탐색

```bash
# 테이블 검색 (오타도 OK)
rds-cli schema show user  # → "users" 제안
rds-cli schema find order # → orders, order_items 찾기

# 관계 분석
rds-cli schema relationships orders
```

### 2. 안전한 쿼리 실행

```bash
# 자동 LIMIT (실수 방지)
rds-cli query "SELECT * FROM orders"
# → SELECT * FROM orders LIMIT 1000

# 프로덕션 읽기 전용
rds-cli --profile prod query "DELETE FROM users"
# → ERROR: Only SELECT queries allowed
```

### 3. Named Queries로 팀 협업

```bash
# .rds-cli.toml에 쿼리 저장 (Git 공유)
rds-cli saved save active_users \
  "SELECT * FROM users WHERE last_login > NOW() - INTERVAL '7 days'"

# 팀원들이 이름으로 실행
rds-cli run active_users

# 파라미터 쿼리
rds-cli saved save find_user "SELECT * FROM users WHERE email = :email"
rds-cli run find_user --param email=test@example.com
```

### 4. 다양한 출력 형식

```bash
# JSON (jq 파이프라인)
rds-cli --format json query "SELECT status, COUNT(*) FROM orders GROUP BY status" \
  | jq '.rows | map({status: .[0], count: .[1]})'

# CSV (엑셀 import)
rds-cli --format csv query "SELECT * FROM products" > products.csv
```

---

## 📦 설치

### 추천: Prebuilt Binary

```bash
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash
```

### Cargo

```bash
cargo install rds-cli
```

**선택사항**: Claude Code Skill을 설치하면 AI가 자연어로 DB를 탐색합니다.

---

## ⚙️ 설정

### 설정 우선순위

```
--profile 옵션 > DB_PASSWORD_<PROFILE> 환경변수 > .rds-cli.toml > ~/.config/rds-cli/config.toml
```

### 최소 설정 예제

**~/.config/rds-cli/config.toml**:

```toml
[defaults]
default_profile = "local"

[profiles.local]
type = "postgresql"
host = "localhost"
port = 5432
user = "myuser"
database = "mydb"

[profiles.local.safety]
default_limit = 1000
allowed_operations = ["SELECT"]
```

**비밀번호는 환경변수로**:

```bash
export DB_PASSWORD_LOCAL="secret"
export DB_PASSWORD_PRODUCTION="prod-secret"
```

**팀 공유 쿼리** (./.rds-cli.toml, Git 커밋):

```toml
[saved_queries.daily_stats]
sql = "SELECT DATE(created_at), COUNT(*) FROM orders GROUP BY 1 ORDER BY 1 DESC LIMIT 7"
description = "최근 7일 주문 통계"
```

### 설정 명령어

```bash
rds-cli config init   # 설정 파일 생성
rds-cli config edit   # $EDITOR로 수정
rds-cli config show   # 현재 설정 확인
```

---

## 💡 실전 활용

### 프로덕션 안전 패턴

```bash
# 프로덕션: 읽기 전용 + 낮은 LIMIT
[profiles.production.safety]
default_limit = 100
max_limit = 1000
allowed_operations = ["SELECT"]

# 개발: 자유롭게
[profiles.dev.safety]
default_limit = 10000
allowed_operations = ["SELECT", "INSERT", "UPDATE", "DELETE"]
```

### 퍼지 검색 활용

```bash
rds-cli schema show user
# ❌ Table 'user' not found
# Did you mean: users, user_roles, user_sessions?
```

### jq 파이프라인

```bash
# Primary key 추출
rds-cli --format json schema show users | jq '.columns[] | select(.is_primary_key)'

# 테이블 이름만
rds-cli --format json schema find order | jq '.tables[].name'
```

---

## 📖 명령어 레퍼런스

| 명령어 | 설명 |
|--------|------|
| `schema find <pattern>` | 테이블 검색 |
| `schema show <table>` | 테이블 상세 조회 |
| `schema relationships <table>` | 관계 분석 |
| `query <sql>` | 쿼리 실행 |
| `run <name> [--param k=v]` | Named query 실행 |
| `saved [save\|delete\|show]` | 쿼리 관리 |
| `refresh` | 스키마 캐시 갱신 |
| `config [init\|edit\|show]` | 설정 관리 |

**공통 옵션**: `--profile <name>`, `--format <json|csv|table>`, `--verbose`

---

## 🛠️ 문제 해결

### "Cache not found" 에러

```bash
rds-cli refresh
```

### "Table not found" 에러

```bash
rds-cli schema find <pattern>  # 테이블 이름 확인
rds-cli refresh                # 캐시 갱신
```

### "Failed to connect" 에러

```bash
# 비밀번호 환경변수 확인
echo $DB_PASSWORD_<PROFILE>

# 연결 테스트
psql -h localhost -U myuser -d mydb  # PostgreSQL
mysql -h localhost -u myuser -p mydb # MySQL
```

---

## 📄 라이선스

MIT OR Apache-2.0

---

**For AI Agents**: [CLAUDE.md](CLAUDE.md)
