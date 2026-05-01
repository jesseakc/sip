'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useAuth } from '@/lib/auth-context';
import { API_BASE } from '@/lib/api';
import {
  LayoutDashboard,
  Package,
  ClipboardList,
  MessageSquare,
  MapPin,
  Users,
  User,
  FileText,
  Settings,
  Menu,
  X,
  LogOut,
  Calendar,
  ClipboardCheck,
  Wrench,
  GitBranch,
  type LucideIcon,
} from 'lucide-react';

// ─── Hardcoded fallback navigation (used when API is unavailable) ──────────────
const FALLBACK_NAV_ITEMS = [
  { id: 'dashboard', label: 'Dashboard', href: '/', icon: 'layout-dashboard' },
  { id: 'assets', label: 'Assets', href: '/assets', icon: 'packages' },
  { id: 'work-orders', label: 'Work Orders', href: '/work-orders', icon: 'clipboard-list' },
  { id: 'schedules', label: 'Schedules', href: '/schedules', icon: 'calendar' },
  { id: 'inspections', label: 'Inspections', href: '/inspections', icon: 'clipboard-check' },
  { id: 'parts', label: 'Parts', href: '/parts', icon: 'wrench' },
  { id: 'ai-chat', label: 'AI Chat', href: '/ai-chat', icon: 'message-square' },
  { id: 'locations', label: 'Locations', href: '/locations', icon: 'map-pin' },
  { id: 'teams', label: 'Teams', href: '/teams', icon: 'users' },
  { id: 'users', label: 'Users', href: '/users', icon: 'user' },
  { id: 'documents', label: 'Documents', href: '/documents', icon: 'file-text' },
  { id: 'settings', label: 'Settings', href: '/settings', icon: 'settings' },
];

// ─── Icon name → Lucide component mapping ─────────────────────────────────────
const ICON_MAP: Record<string, LucideIcon> = {
  'layout-dashboard': LayoutDashboard,
  'packages': Package,
  'package': Package,
  'boxes': Package,
  'clipboard-list': ClipboardList,
  'calendar': Calendar,
  'clipboard-check': ClipboardCheck,
  'wrench': Wrench,
  'message-square': MessageSquare,
  'map-pin': MapPin,
  'users': Users,
  'user': User,
  'file-text': FileText,
  'settings': Settings,
  'git-branch': GitBranch,
};

function resolveIcon(iconName: string | null | undefined): LucideIcon {
  if (!iconName) return LayoutDashboard;
  return ICON_MAP[iconName] || LayoutDashboard;
}

// ─── API NavItem type ──────────────────────────────────────────────────────────
interface ApiNavItem {
  id: string;
  label: string;
  path: string;
  icon?: string | null;
  permission?: string | null;
  order?: number | null;
}

export default function Shell({ children }: { children: React.ReactNode }) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [navItems, setNavItems] = useState(FALLBACK_NAV_ITEMS);
  const [navLoading, setNavLoading] = useState(true);
  const pathname = usePathname();
  const { user, logout } = useAuth();

  // Fetch navigation from the API when the user is authenticated.
  // Falls back to FALLBACK_NAV_ITEMS if the request fails or times out.
  useEffect(() => {
    if (!user) {
      setNavLoading(false);
      return;
    }

    const token = localStorage.getItem('sip_token');
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    fetch(`${API_BASE}/ui/navigation`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      signal: controller.signal,
    })
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
      })
      .then((json) => {
        const data = json?.data;
        if (Array.isArray(data) && data.length > 0) {
          // Convert API response to the format expected by the renderer
          const items = data.map((item: ApiNavItem) => ({
            id: item.id,
            label: item.label,
            href: item.path,
            icon: item.icon || 'layout-dashboard',
          }));
          setNavItems(items);
        }
        setNavLoading(false);
      })
      .catch(() => {
        // API unavailable — fall back to defaults
        setNavLoading(false);
      })
      .finally(() => clearTimeout(timeout));

    return () => {
      controller.abort();
      clearTimeout(timeout);
    };
  }, [user]);

  if (pathname === '/login') {
    return <>{children}</>;
  }

  return (
    <div className="min-h-screen bg-gray-50 flex">
      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={() => setMobileOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={`fixed inset-y-0 left-0 z-50 w-64 bg-white border-r border-gray-200 transform transition-transform duration-200 ease-in-out lg:translate-x-0 lg:static lg:inset-auto lg:z-auto ${
          mobileOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
        <div className="flex items-center justify-between h-16 px-6 border-b border-gray-200">
          <span className="text-xl font-bold text-blue-600">SIP</span>
          <button
            onClick={() => setMobileOpen(false)}
            className="lg:hidden p-2 rounded-md hover:bg-gray-100"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        <nav className="p-4 space-y-1">
          {navItems.map((item) => {
            const Icon = resolveIcon(item.icon);
            const isActive =
              item.href === '/'
                ? pathname === '/'
                : pathname === item.href || pathname.startsWith(`${item.href}/`);
            return (
              <Link
                key={item.id}
                href={item.href}
                onClick={() => setMobileOpen(false)}
                className={`flex items-center space-x-3 px-3 py-2 rounded-md text-sm font-medium ${
                  isActive
                    ? 'bg-blue-50 text-blue-700'
                    : 'text-gray-700 hover:bg-gray-50'
                }`}
              >
                <Icon className="w-5 h-5" />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </aside>

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Top bar */}
        <header className="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-4 lg:px-8">
          <button
            onClick={() => setMobileOpen(true)}
            className="lg:hidden p-2 rounded-md hover:bg-gray-100"
          >
            <Menu className="w-5 h-5" />
          </button>
          <div className="flex items-center space-x-4 ml-auto">
            {user && (
              <>
                <span className="text-sm text-gray-700 hidden sm:block">
                  {user.name}
                </span>
                <button
                  onClick={logout}
                  className="flex items-center space-x-1 text-sm text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md hover:bg-gray-100"
                >
                  <LogOut className="w-4 h-4" />
                  <span className="hidden sm:inline">Logout</span>
                </button>
              </>
            )}
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 p-4 lg:p-8 overflow-auto">
          {children}
        </main>
      </div>
    </div>
  );
}
