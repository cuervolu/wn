type Theme = 'cerberus' | 'mona';

function createThemeStore() {
	const saved =
		typeof localStorage !== 'undefined' ? (localStorage.getItem('wn-theme') as Theme | null) : null;

	let current = $state<Theme>(saved ?? 'cerberus');

	function set(theme: Theme) {
		current = theme;
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('wn-theme', theme);
		}
	}

	return {
		get current() {
			return current;
		},
		set,
		toggle() {
			set(current === 'cerberus' ? 'mona' : 'cerberus');
		},
		// cerberus es el tema oscuro, mona el claro
		get isDark() {
			return current === 'cerberus';
		}
	};
}

export const themeStore = createThemeStore();
