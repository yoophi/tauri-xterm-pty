# Project Guide

## Frontend architecture

- `src`는 Feature-Sliced Design을 따른다.
- 앱 조합은 `app`, 화면은 `pages`, 사용자 상호작용은 `features`, 도메인 모델과 API는
  `entities`, 범용 코드는 `shared`에 둔다.
- 의존 방향은 `app → pages → features → entities → shared`로 유지한다.
- shadcn/ui registry 컴포넌트는 `src/components/ui`에 둔다.
- 한 화면에만 의미가 있는 코드를 성급하게 `shared`로 올리지 않는다.

## Rust architecture

- `src-tauri/src`는 헥사고날 아키텍처를 따른다.
- 순수 도메인 모델은 `domain`, 유스케이스는 `application`, outbound port trait은 `ports`에 둔다.
- Tauri command 같은 inbound adapter는 `inbound`, OS·PTY·파일시스템 구현은
  `infrastructure`에 둔다.
- `domain`과 `ports`는 Tauri나 구체적인 OS API에 의존하지 않는다.
- Tauri command는 비즈니스 로직을 직접 구현하지 않고 application service에 위임한다.

## Verification

- 프런트엔드: `npm run check-types`, `npm test`, `npm run build`
- Rust: `cargo fmt --check --manifest-path src-tauri/Cargo.toml`,
  `cargo test --manifest-path src-tauri/Cargo.toml`
- 데스크탑 통합: `npm run tauri build -- --debug --no-bundle`

## Documentation

- 문서 본문은 한국어로 작성한다.
- 다이어그램은 Mermaid 문법을 사용한다.
