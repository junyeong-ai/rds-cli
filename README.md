# RDS CLI

[![CI](https://github.com/junyeong-ai/rds-cli/workflows/CI/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Lint](https://github.com/junyeong-ai/rds-cli/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Rust](https://img.shields.io/badge/rust-1.91.1%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/junyeong-ai/rds-cli/releases)
[![DeepWiki](https://img.shields.io/badge/DeepWiki-junyeong--ai%2Frds--cli-blue.svg?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAyCAYAAAAnWDnqAAAAAXNSR0IArs4c6QAAA05JREFUaEPtmUtyEzEQhtWTQyQLHNak2AB7ZnyXZMEjXMGeK/AIi+QuHrMnbChYY7MIh8g01fJoopFb0uhhEqqcbWTp06/uv1saEDv4O3n3dV60RfP947Mm9/SQc0ICFQgzfc4CYZoTPAswgSJCCUJUnAAoRHOAUOcATwbmVLWdGoH//PB8mnKqScAhsD0kYP3j/Yt5LPQe2KvcXmGvRHcDnpxfL2zOYJ1mFwrryWTz0advv1Ut4CJgf5uhDuDj5eUcAUoahrdY/56ebRWeraTjMt/00Sh3UDtjgHtQNHwcRGOC98BJEAEymycmYcWwOprTgcB6VZ5JK5TAJ+fXGLBm3FDAmn6oPPjR4rKCAoJCal2eAiQp2x0vxTPB3ALO2CRkwmDy5WohzBDwSEFKRwPbknEggCPB/imwrycgxX2NzoMCHhPkDwqYMr9tRcP5qNrMZHkVnOjRMWwLCcr8ohBVb1OMjxLwGCvjTikrsBOiA6fNyCrm8V1rP93iVPpwaE+gO0SsWmPiXB+jikdf6SizrT5qKasx5j8ABbHpFTx+vFXp9EnYQmLx02h1QTTrl6eDqxLnGjporxl3NL3agEvXdT0WmEost648sQOYAeJS9Q7bfUVoMGnjo4AZdUMQku50McDcMWcBPvr0SzbTAFDfvJqwLzgxwATnCgnp4wDl6Aa+Ax283gghmj+vj7feE2KBBRMW3FzOpLOADl0Isb5587h/U4gGvkt5v60Z1VLG8BhYjbzRwyQZemwAd6cCR5/XFWLYZRIMpX39AR0tjaGGiGzLVyhse5C9RKC6ai42ppWPKiBagOvaYk8lO7DajerabOZP46Lby5wKjw1HCRx7p9sVMOWGzb/vA1hwiWc6jm3MvQDTogQkiqIhJV0nBQBTU+3okKCFDy9WwferkHjtxib7t3xIUQtHxnIwtx4mpg26/HfwVNVDb4oI9RHmx5WGelRVlrtiw43zboCLaxv46AZeB3IlTkwouebTr1y2NjSpHz68WNFjHvupy3q8TFn3Hos2IAk4Ju5dCo8B3wP7VPr/FGaKiG+T+v+TQqIrOqMTL1VdWV1DdmcbO8KXBz6esmYWYKPwDL5b5FA1a0hwapHiom0r/cKaoqr+27/XcrS5UwSMbQAAAABJRU5ErkJggg==)](https://deepwiki.com/junyeong-ai/rds-cli)

> **🌐 한국어** | **[English](README.en.md)**

---

> **⚡ PostgreSQL/MySQL을 위한 빠르고 안전한 Database CLI**
>
> - 🚀 **초고속** (Rust 기반, <5ms 스키마 조회)
> - 🔒 **프로덕션 안전** (자동 LIMIT, 읽기 전용)
> - 📝 **팀 협업** (Git 버전 관리 Named Queries)
> - 🔍 **스마트 검색** (퍼지 매칭, 자동 완성)

---

## 핵심 기능

- **빠른 스키마 조회**: 캐싱으로 <5ms
- **안전한 쿼리**: 자동 LIMIT, 읽기 전용 모드
- **팀 협업**: Git 버전 관리 Named Queries
- **암호화 비밀번호**: Git 안전, 환경변수 불필요
- **스마트 검색**: 퍼지 매칭, 자동 제안

---

## ⚡ 빠른 시작

```bash
# 1. 설치 (전역 설정 자동 생성)
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash

# 2. 프로젝트 설정
cd your-project
rds-cli config init     # .rds-cli.toml 생성
rds-cli config edit     # DB 정보 입력

# 3. 비밀번호 설정 (암호화)
rds-cli secret set local

# 4. 스키마 캐싱 및 사용
rds-cli refresh
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

### 3. 암호화된 비밀번호 관리

```bash
# 비밀번호 설정 (암호화되어 .rds-cli.toml에 저장)
rds-cli secret set production

# 자동화
echo "password" | rds-cli secret set production --password-stdin
```

### 4. Named Queries로 팀 협업

```bash
# .rds-cli.toml에 쿼리 저장 (Git 공유)
rds-cli saved save active_users \
  "SELECT * FROM users WHERE last_login > NOW() - INTERVAL '7 days'"

# 팀원들이 이름으로 실행
rds-cli run active_users

# 파라미터 쿼리
rds-cli saved save find_user "SELECT * FROM users WHERE email = :email"
rds-cli run find_user --arg email=test@example.com
```

### 5. 다양한 출력 형식

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

### 설정 구조

**전역 설정** (`~/.config/rds-cli/config.toml`, 설치 시 자동 생성):
```toml
[defaults]
default_profile = "local"
cache_ttl_hours = 24
output_format = "table"
```

**프로젝트 설정** (`.rds-cli.toml`, `config init`으로 생성):
```toml
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

**우선순위**: CLI args > 암호화 비밀번호 > 환경변수 > 프로젝트 설정 > 전역 설정

### 비밀번호 관리

**권장: 암호화 저장**
```bash
rds-cli secret set local
# .rds-cli.toml에 암호화되어 저장 (Git 안전)
```

**선택: 환경변수**
```bash
export DB_PASSWORD_LOCAL="secret"
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
rds-cli config path   # 설정 파일 경로 출력
```

---

## 프로덕션 설정

```toml
[profiles.production.safety]
default_limit = 100
max_limit = 1000
allowed_operations = ["SELECT"]  # 읽기 전용
```

---

## 📖 명령어 레퍼런스

| 명령어 | 설명 |
|--------|------|
| `schema find <pattern>` | 테이블 검색 |
| `schema show <table>` | 테이블 상세 조회 |
| `schema relationships <table>` | 관계 분석 |
| `query <sql>` | 쿼리 실행 |
| `run <name> [-a k=v]` | Named query 실행 |
| `saved [list\|save\|delete\|show]` | 쿼리 관리 |
| `secret set <profile>` | 비밀번호 암호화 저장 |
| `secret get <profile>` | 비밀번호 복호화 출력 |
| `secret remove <profile>` | 비밀번호 제거 |
| `secret reset` | 마스터 키 초기화 |
| `refresh` | 스키마 캐시 갱신 |
| `config [init\|edit\|show\|path]` | 설정 관리 |

**옵션**: `--profile <name>`, `--format <json|csv|table>`, `--verbose`

---

## 문제 해결

```bash
# 캐시 없음
rds-cli refresh

# 연결 실패
rds-cli secret get <profile>

# 마스터 키 분실
rds-cli secret reset
rds-cli secret set <profile>
```

---

## 📄 라이선스

MIT OR Apache-2.0

---

**For AI Agents**: [CLAUDE.md](CLAUDE.md)
