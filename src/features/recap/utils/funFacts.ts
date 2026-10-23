import { formatPlaytime } from "@/utils/format";

/** Generates a fun fact comparing playtime to movies/pop culture references. */
export function getPlaytimeFact(hours: number, locale: "ru" | "en"): string {
  const lordOfTheRings = 11.4; // hours for extended edition
  const movies = Math.floor(hours / 2); // average movie ~2h

  const facts: Record<string, string[]> = {
    ru: [
      hours >= 1000
        ? `Это как посмотреть все серии Игры Престолов ${Math.floor(hours / 9)} раз`
        : hours >= 100
          ? `Это как пересмотреть Властелина Колец ${Math.floor(hours / lordOfTheRings)} раз`
          : hours >= 10
            ? `Вы посмотрели бы ${movies} полнометражных фильмов`
            : "Самое начало великого пути!",
      hours >= 500
        ? "Вы — легендарный геймер с тысячами часов за плечами"
        : hours >= 100
          ? "Сотни часов в играх — это серьёзное погружение!"
          : "Каждый час приближает вас к статусу хардкорного геймера",
      `Вы могли бы пройти ${Math.floor(hours / 20)} полноценных RPG средней продолжительности`,
    ],
    en: [
      hours >= 1000
        ? `That's like watching Game of Thrones ${Math.floor(hours / 9)} times`
        : hours >= 100
          ? `You could rewatch Lord of the Rings ${Math.floor(hours / lordOfTheRings)} times`
          : hours >= 10
            ? `You could watch ${movies} full-length movies`
            : "The beginning of a great journey!",
      hours >= 500
        ? "You're a legendary gamer with thousands of hours behind you"
        : hours >= 100
          ? "Hundreds of hours in games — that's serious dedication!"
          : "Every hour brings you closer to hardcore status",
      `You could complete ${Math.floor(hours / 20)} medium-length RPGs`,
    ],
  };

  const localeFacts = facts[locale] ?? facts.ru ?? [];
  return localeFacts[Math.floor(Math.random() * localeFacts.length)] ?? "";
}

/** Get genre-related tagline. */
export function getGenreTagline(genre: string | null, locale: "ru" | "en"): string {
  const taglines: Record<string, Record<string, string>> = {
    ru: {
      RPG: "настоящий стратег и исследователь",
      "Action RPG": "мастер боя и приключений",
      Shooter: "меткий стрелок с твёрдой рукой",
      FPS: "холоднокровный снайпер",
      TPS: "тактический гений перестрелок",
      Action: "ценитель экшена и адреналина",
      Adventure: "исследователь неизведанных миров",
      "Horror": "любитель пощекотать нервы",
      Strategy: "мастер стратегического мышления",
      Simulation: "повелитель симуляций",
      Racing: "король скорости и драйва",
      Sports: "спортивный энтузиаст",
      "Visual Novel": "ценитель интерактивных историй",
      Platformer: "мастер прыжков и платформинга",
      Puzzle: "гений логики и загадок",
      MMO: "социальный магистр онлайн-мира",
      MOBA: "легенда полей боя",
    },
    en: {
      RPG: "a true strategist and explorer",
      "Action RPG": "a master of combat and adventure",
      Shooter: "a sharpshooter with a steady hand",
      FPS: "a cold-blooded sniper",
      TPS: "a tactical genius of shootouts",
      Action: "an adrenaline junkie",
      Adventure: "an explorer of uncharted worlds",
      Horror: "someone who likes to get scared",
      Strategy: "a strategic mastermind",
      Simulation: "a simulation lord",
      Racing: "a speed and drift king",
      Sports: "a sports enthusiast",
      "Visual Novel": "a fan of interactive stories",
      Platformer: "a platforming master",
      Puzzle: "a logic genius",
      MMO: "a social online master",
      MOBA: "a legend of the battlefield",
    },
  };

  if (!genre) return locale === "ru" ? "универсальный геймер" : "a versatile gamer";
  const dict = taglines[locale] ?? taglines.ru ?? {};
  return dict[genre] ?? (locale === "ru" ? "настоящий ценитель жанра" : "a true genre enthusiast");
}

/** Format seconds to hours string. */
export function formatHours(seconds: number): string {
  const hours = seconds / 3600;
  if (hours < 1) return `${Math.round(hours * 60)} ${formatPlaytime(seconds)}`.split(" ")[0] + " мин";
  return `${Math.round(hours)} ч`;
}
