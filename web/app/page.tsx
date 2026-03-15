import Navbar from '@/components/docs/Navbar'
import HomeHero from '@/components/landing/HomeHero'
import HomeFeatures from '@/components/landing/HomeFeatures'
import HomeDemo from '@/components/landing/HomeDemo'
import HomeInstall from '@/components/landing/HomeInstall'
import HomeCmux from '@/components/landing/HomeCmux'
import HomeFooter from '@/components/landing/HomeFooter'

export default function HomePage() {
  return (
    <main>
      <Navbar />
      <HomeHero />
      <HomeFeatures />
      <HomeDemo />
      <HomeInstall />
      <HomeCmux />
      <HomeFooter />
    </main>
  )
}
