/**
 * Test flags for manual scenario checks. Flip them off before release.
 *
 * TEST: пока `true` при каждом входе в приложение показывается окно ввода
 * ника — после подтверждения/закрытия значение сохраняется в БД
 * (`settings.player_name`) и сразу отображается на клиенте (топбар).
 * Production-поведение соответствует `false`: окно открывается только
 * на онбординге при первой инициализации.
 */
export const SHOW_NAME_PROMPT_FOR_TEST = false;
