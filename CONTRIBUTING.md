# Contributing to LostLove Protocol

Спасибо за интерес к участию в разработке LostLove Protocol! Этот документ содержит рекомендации по внесению вклада в проект.

## Кодекс поведения

Участвуя в этом проекте, вы соглашаетесь соблюдать наш кодекс поведения:

- Будьте уважительны к другим участникам
- Принимайте конструктивную критику
- Фокусируйтесь на том, что лучше для сообщества
- Проявляйте эмпатию к другим участникам

## Как внести вклад

### Reporting Bugs

Перед созданием bug report:
- Проверьте, не была ли уже создана подобная issue
- Убедитесь, что проблема воспроизводима

При создании bug report укажите:
- Описание проблемы
- Шаги для воспроизведения
- Ожидаемое поведение
- Фактическое поведение
- Версию ПО
- Операционную систему
- Логи (если применимо)

### Suggesting Features

При предложении новой функциональности:
- Опишите проблему, которую решает функция
- Предложите решение
- Опишите альтернативные решения
- Укажите, как это влияет на производительность/безопасность

### Pull Requests

1. **Fork репозиторий**
   ```bash
   git clone https://github.com/Salamander5876/LostLove-Protocol.git
   cd LostLove-Protocol
   ```

2. **Создайте feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```

3. **Внесите изменения**
   - Следуйте code style проекта
   - Добавьте тесты для новой функциональности
   - Обновите документацию

4. **Commit изменений**
   ```bash
   git commit -m "feat: add amazing feature"
   ```

   Используйте [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` - новая функциональность
   - `fix:` - исправление бага
   - `docs:` - изменения в документации
   - `style:` - форматирование кода
   - `refactor:` - рефакторинг
   - `test:` - добавление тестов
   - `chore:` - обновление зависимостей и т.д.

5. **Push в свой fork**
   ```bash
   git push origin feature/amazing-feature
   ```

6. **Создайте Pull Request**
   - Опишите изменения
   - Укажите связанные issues
   - Приложите скриншоты (если применимо)

## Требования к коду

### Rust (Server)

```rust
// Используйте clippy
cargo clippy -- -D warnings

// Форматирование
cargo fmt

// Тесты
cargo test

// Бенчмарки
cargo bench
```

**Code Style**:
- Следуйте [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Документируйте публичные API
- Используйте `rustfmt` для форматирования
- Избегайте `unwrap()` в production коде

### TypeScript (Client GUI)

```bash
# Линтинг
npm run lint

# Форматирование
npm run format

# Тесты
npm test

# Type checking
npm run type-check
```

**Code Style**:
- Используйте TypeScript strict mode
- Документируйте сложные функции
- Следуйте [Google TypeScript Style Guide](https://google.github.io/styleguide/tsguide.html)

### C++ (Client Service)

```cpp
// Используйте clang-format
clang-format -i src/**/*.cpp

// Статический анализ
clang-tidy src/**/*.cpp
```

**Code Style**:
- Следуйте [C++ Core Guidelines](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines)
- Используйте modern C++ (C++17+)
- RAII для управления ресурсами
- Избегайте raw pointers

## Тестирование

### Unit Tests

Все новые функции должны иметь unit тесты:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = create_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

Для сложных изменений добавьте integration тесты:

```rust
#[tokio::test]
async fn test_end_to_end_flow() {
    // Setup
    let server = start_test_server().await;
    let client = connect_test_client().await;

    // Execute
    let result = perform_operation(&client).await;

    // Verify
    assert!(result.is_ok());

    // Cleanup
    server.shutdown().await;
}
```

### Performance Tests

Для изменений, влияющих на производительность:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_feature(c: &mut Criterion) {
    c.bench_function("feature name", |b| {
        b.iter(|| {
            // Code to benchmark
            function_to_test(black_box(input))
        });
    });
}

criterion_group!(benches, benchmark_feature);
criterion_main!(benches);
```

## Документация

### Code Documentation

```rust
/// Encrypts data using QuantumShield triple-layer encryption.
///
/// # Arguments
///
/// * `plaintext` - The data to encrypt
/// * `keys` - The encryption keys
///
/// # Returns
///
/// Returns encrypted data or an error if encryption fails.
///
/// # Examples
///
/// ```
/// let keys = generate_keys();
/// let encrypted = encrypt_data(&data, &keys)?;
/// ```
pub fn encrypt_data(plaintext: &[u8], keys: &Keys) -> Result<Vec<u8>> {
    // Implementation
}
```

### User Documentation

При добавлении новых возможностей обновите:
- README.md - если изменяется API или установка
- docs/ - детальную документацию
- CHANGELOG.md - список изменений

## Security

### Reporting Security Issues

**НЕ создавайте публичные issues для уязвимостей безопасности!**

Вместо этого:
1. Отправьте email на security@lostlove.io
2. Опишите уязвимость
3. Укажите шаги для воспроизведения
4. Предложите решение (если есть)

### Security Requirements

- Все новые криптографические функции должны быть проверены экспертами
- Используйте проверенные криптографические библиотеки
- Избегайте custom crypto без консультации с экспертами
- Все секреты должны быть в secure storage
- Никогда не логируйте чувствительные данные

## Процесс Review

### Для контрибьюторов

1. Создайте PR с описанием изменений
2. Убедитесь, что все тесты проходят
3. Ответьте на комментарии reviewers
4. Внесите запрошенные изменения
5. Дождитесь approval

### Для reviewers

1. Проверьте соответствие code style
2. Убедитесь в наличии тестов
3. Проверьте документацию
4. Рассмотрите влияние на производительность
5. Проверьте безопасность
6. Оставьте конструктивные комментарии

## Релиз процесс

1. Обновите CHANGELOG.md
2. Обновите версию в Cargo.toml / package.json
3. Создайте git tag
4. Создайте release на GitHub
5. Соберите и опубликуйте binaries

## Полезные ресурсы

### Documentation
- [Архитектура проекта](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Implementation Guide](docs/IMPLEMENTATION_GUIDE.md)

### Learning
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)

### Tools
- [Rust Playground](https://play.rust-lang.org/)
- [TypeScript Playground](https://www.typescriptlang.org/play)
- [Compiler Explorer](https://godbolt.org/)

## Вопросы?

Если у вас есть вопросы:
- Создайте issue с меткой `question`
- Присоединяйтесь к обсуждениям на GitHub Discussions
- Проверьте существующую документацию

## Благодарности

Спасибо всем контрибьюторам, которые помогают развивать проект!

---

**Помните**: Качество важнее количества. Лучше один хорошо протестированный PR, чем десять непроверенных.
