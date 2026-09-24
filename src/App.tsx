import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { MotionConfig } from "framer-motion";
import { AppLayout } from "@/components/layout/AppLayout";
import { HomePage } from "@/pages/HomePage";
import { LibraryPage } from "@/pages/LibraryPage";
import { GameDetailsPage } from "@/pages/GameDetailsPage";
import { AchievementsPage } from "@/pages/AchievementsPage";
import { StatisticsPage } from "@/pages/StatisticsPage";
import { FriendsPage } from "@/pages/FriendsPage";
import { ShopPage } from "@/pages/ShopPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { useSettingsStore } from "@/stores/settingsStore";

export default function App() {
  const animations = useSettingsStore((s) => s.animations);

  return (
    <MotionConfig reducedMotion={animations ? "user" : "always"}>
      <HashRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/" element={<HomePage />} />
            <Route path="/library" element={<LibraryPage />} />
            <Route path="/game/:id" element={<GameDetailsPage />} />
            <Route path="/achievements" element={<AchievementsPage />} />
            <Route path="/statistics" element={<StatisticsPage />} />
            <Route path="/friends" element={<FriendsPage />} />
            <Route path="/shop" element={<ShopPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </HashRouter>
    </MotionConfig>
  );
}
