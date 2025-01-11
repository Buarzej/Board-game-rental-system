# System wypożyczania planszówek

## Backend (Rust)

### Podstawowe funkcjonalności

- Obsługa rejestracji użytkowników
    - Haszowanie haseł użytkowników za pomocą algorytmu Argon2
    - Weryfikacja adresu email przy rejestracji konta
    - Generowanie tokenów JWT w celu utrzymania sesji logowania
    - Weryfikacja uprawnień administratora przy wykonywaniu odpowiednich operacji
- Konfiguracja danych wrażliwych w pliku `.env`
- Zarządzanie bazą danych
    - Obsługa wprowadzania zmian w strukturze bazy danych (moduł `migration`)
    - Mapowanie struktury bazy danych na struktury języka Rust (moduł `entity`)
    - Obsługa operacji CRUD na bazie danych
- Udostępnienie _API endpoints_ pozwalających na wysyłanie zapytań HTTP
    - Obsługa zapytań typu GET, POST, PUT, DELETE
    - Przesyłanie plików na serwer i statyczne ich serwowanie

### Testy

Niestety, repozytorium nie posiada testów jednostkowych (z powodu braku czasu na ich stworzenie -- w niedalekiej
przyszłości z pewnością powstaną).

Tymczasowo, na poczet ręcznego testowania, udostępnione zostały trzy endpointy:
- `GET /login` -- zwraca formularz logowania
- `GET /register` -- zwraca formularz rejestracji
- `GET /board_game` -- zwraca formularz dodawania nowej gry planszowej

Jest to rozwiązanie mocno robocze i zostanie zastąpione w momencie utworzenia testów jednostkowych.

### Uruchomienie

1. Sklonuj repozytorium
2. Zainstaluj zależności za pomocą `cargo build`
3. Skonfiguruj plik `.env` (przykład znajduje się w pliku `.env.example`)
4. Uruchom serwer za pomocą `cargo run`
5. Serwer domyślnie działa pod adresem `http://localhost:8080`

### Planowane zmiany i rozwinięcia

- Dodanie testów jednostkowych
- Zwiększenie bezpieczeństwa aplikacji
    - Nadanie tokenom rejestracyjnym czasu ważności
    - Ograniczenie liczby prób logowania i rejestracji
    - Szyfrowanie bazy danych
    - Obsługa protokołu HTTPS
- Implementacja funkcji "zapamiętaj mnie"
- Dodanie funkcji "zapomniałem hasła"
- Obsługa powiadomień mailowych wysyłanych przez system do użytkowników