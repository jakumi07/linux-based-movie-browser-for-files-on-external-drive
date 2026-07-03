import { invoke } from "@tauri-apps/api/core";

interface Movie {
  name: string;
  path: string;
  folder: string;
}

// Function to generate a stable, nice looking background color from a folder name
function getFolderColor(folder: string): string {
  if (!folder) return "#555555";
  let hash = 0;
  for (let i = 0; i < folder.length; i++) {
    hash = folder.charCodeAt(i) + ((hash << 5) - hash);
  }
  // Convert hash to a warm/vibrant pastel HSL color
  const h = Math.abs(hash % 360);
  return `hsl(${h}, 70%, 40%)`;
}

window.addEventListener("DOMContentLoaded", () => {
  const scanBtn = document.querySelector<HTMLButtonElement>("#scan-btn");
  const searchInput = document.querySelector<HTMLInputElement>("#search-input");
  const movieContainer = document.querySelector("#movie-container");
  const statusMessage = document.querySelector("#status-message");

  if (!scanBtn || !searchInput || !movieContainer || !statusMessage) return;

  let allMovies: Movie[] = [];
  let allSections: HTMLElement[] = [];

  const renderResults = () => {
    const query = searchInput.value.trim().toLowerCase();
    let visibleCount = 0;

    allSections.forEach((section) => {
      const cards = Array.from(section.querySelectorAll<HTMLElement>(".movie-card"));
      let sectionVisibleCount = 0;

      cards.forEach((card) => {
        const text = card.dataset.searchText || "";
        const matches = !query || text.includes(query);
        card.classList.toggle("is-hidden", !matches);
        if (matches) {
          sectionVisibleCount += 1;
        }
      });

      const header = section.querySelector<HTMLElement>(".folder-header");
      const countBadge = section.querySelector<HTMLElement>(".folder-count");
      const visibleCards = cards.filter((card) => !card.classList.contains("is-hidden"));

      if (query && visibleCards.length === 0) {
        section.classList.add("is-hidden");
        sectionVisibleCount = 0;
      } else {
        section.classList.remove("is-hidden");
      }

      if (header && countBadge) {
        const folderName = header.dataset.folderName || "";
        const visibleLabel = `${visibleCards.length} of ${cards.length} movie${cards.length === 1 ? "" : "s"}`;
        countBadge.textContent = query ? visibleLabel : `${cards.length} movie${cards.length === 1 ? "" : "s"}`;
        header.querySelector<HTMLElement>(".folder-title")!.textContent = folderName;
      }

      visibleCount += sectionVisibleCount;
    });

    if (query && visibleCount === 0) {
      const emptyState = document.createElement("div");
      emptyState.className = "empty-state";
      emptyState.innerHTML = `<strong>No matches</strong><div>Try a different title, folder, or path.</div>`;
      movieContainer.querySelector(".empty-state")?.remove();
      movieContainer.appendChild(emptyState);
    } else {
      movieContainer.querySelector(".empty-state")?.remove();
    }

    statusMessage.textContent = query
      ? `Showing ${visibleCount} matching movie${visibleCount === 1 ? "" : "s"}.`
      : allMovies.length > 0
        ? `Found ${allMovies.length} movies!`
        : "Click \"Scan Drive\" to search for movies.";
  };

  searchInput.addEventListener("input", renderResults);

  const showLoadingState = () => {
    movieContainer.innerHTML = "";
    const shimmerCount = 6;
    for (let i = 0; i < shimmerCount; i += 1) {
      const shimmer = document.createElement("div");
      shimmer.className = "shimmer-card";
      movieContainer.appendChild(shimmer);
    }
  };

  scanBtn.addEventListener("click", async () => {
    scanBtn.disabled = true;
    scanBtn.textContent = "Scanning...";
    statusMessage.textContent = "Scanning for movies...";
    searchInput.value = "";
    movieContainer.innerHTML = "";
    showLoadingState();

    try {
      const movies = await invoke<Movie[]>("scan_movies");

      if (!Array.isArray(movies)) {
        throw new Error("The scan response was invalid.");
      }

      allMovies = movies;
      movieContainer.innerHTML = "";

      if (movies.length === 0) {
        statusMessage.textContent = "No playable movies were found. Create a Movies folder and add video files to scan them.";
        return;
      }

      const groupedMovies: { [key: string]: Movie[] } = {};
      movies.forEach((movie) => {
        const folderName = movie.folder || "General";
        if (!groupedMovies[folderName]) {
          groupedMovies[folderName] = [];
        }
        groupedMovies[folderName].push(movie);
      });

      const sortedFolders = Object.keys(groupedMovies).sort((a, b) => {
        if (a === "General") return -1;
        if (b === "General") return 1;
        return a.localeCompare(b);
      });

      allSections = [];

      sortedFolders.forEach((folderName) => {
        const folderMovies = groupedMovies[folderName];
        const section = document.createElement("div");
        section.className = "folder-section";

        const header = document.createElement("div");
        header.className = "folder-header";
        header.dataset.folderName = folderName;

        const icon = document.createElement("span");
        icon.className = "folder-icon";
        icon.textContent = "📁";

        const title = document.createElement("span");
        title.className = "folder-title";
        title.textContent = folderName;

        const count = document.createElement("span");
        count.className = "folder-count";
        count.textContent = `${folderMovies.length} movie${folderMovies.length > 1 ? "s" : ""}`;

        header.appendChild(icon);
        header.appendChild(title);
        header.appendChild(count);
        section.appendChild(header);

        const grid = document.createElement("div");
        grid.className = "movie-grid";

        folderMovies.forEach((movie) => {
          const card = document.createElement("div");
          card.className = "movie-card";
          const searchText = `${movie.name} ${movie.folder} ${movie.path}`.toLowerCase();
          card.dataset.searchText = searchText;

          const thumb = document.createElement("div");
          thumb.className = "movie-card-thumb";
          thumb.textContent = "🎬";

          const info = document.createElement("div");
          info.className = "movie-card-info";

          const movieTitle = document.createElement("div");
          movieTitle.className = "movie-title";
          movieTitle.textContent = movie.name;

          const badgeContainer = document.createElement("div");
          badgeContainer.className = "movie-badge-container";

          const folderBadge = document.createElement("span");
          folderBadge.className = "movie-badge";
          folderBadge.textContent = folderName;
          folderBadge.style.backgroundColor = getFolderColor(movie.folder || folderName);

          badgeContainer.appendChild(folderBadge);

          const path = document.createElement("div");
          path.className = "movie-path";
          path.textContent = movie.path;

          info.appendChild(movieTitle);
          info.appendChild(badgeContainer);
          info.appendChild(path);
          card.appendChild(thumb);
          card.appendChild(info);

          card.addEventListener("click", async () => {
            try {
              await invoke("play_movie", { path: movie.path });
            } catch (playErr) {
              statusMessage.textContent = `Could not open movie: ${playErr}`;
            }
          });

          grid.appendChild(card);
        });

        section.appendChild(grid);
        movieContainer.appendChild(section);
        allSections.push(section);
      });

      renderResults();
    } catch (err) {
      statusMessage.textContent = `Error: ${err}`;
    } finally {
      scanBtn.disabled = false;
      scanBtn.textContent = "Scan Drive";
    }
  });
});
