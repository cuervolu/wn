import { createFileRoute } from '@tanstack/react-router';
import { LandingHomePage } from '@/components/landing/home-page';

export const Route = createFileRoute('/')({
  component: Home,
});

function Home() {
  return <LandingHomePage />;
}
