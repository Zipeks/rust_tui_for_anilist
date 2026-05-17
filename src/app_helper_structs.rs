#[derive(PartialEq)]
pub enum ActiveBlock {
    Sidebar,
    Center,
    Details,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CurrentView {
    UserAnime,
    UserManga,
    BrowseAnime,
    BrowseManga,
}

impl CurrentView {
    pub const ALL: [CurrentView; 4] = [
        CurrentView::UserAnime,
        CurrentView::UserManga,
        CurrentView::BrowseAnime,
        CurrentView::BrowseManga,
    ];
}
impl std::fmt::Display for CurrentView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CurrentView::UserAnime => "Anime",
            CurrentView::UserManga => "Manga",
            CurrentView::BrowseAnime => "Browse Anime",
            CurrentView::BrowseManga => "Browse Manga",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub allows_nsfw: Option<bool>,
}

impl User {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
pub struct PageInfo {
    pub current_page: i64,
    pub per_page: i64,
    pub total: Option<i64>,
    pub last_page: Option<i64>,
    pub has_next_page: Option<bool>,
}

use ratatui::widgets::TableState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
    Unknown,
}
impl MediaStatus {
    pub const ALL: [MediaStatus; 6] = [
        MediaStatus::Current,
        MediaStatus::Planning,
        MediaStatus::Completed,
        MediaStatus::Dropped,
        MediaStatus::Paused,
        MediaStatus::Repeating,
    ];
}

impl std::fmt::Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MediaStatus::Current => "Oglądane",
            MediaStatus::Planning => "W planach",
            MediaStatus::Completed => "Ukończone",
            MediaStatus::Dropped => "Porzucone",
            MediaStatus::Paused => "Wstrzymane",
            MediaStatus::Repeating => "Oglądane ponownie",
            MediaStatus::Unknown => "Nieznany",
        };
        write!(f, "{}", s)
    }
}
use crate::anilist::{
    get_media,
    get_user_media_list::{self, MediaListStatus},
};
impl From<get_user_media_list::MediaListStatus> for MediaStatus {
    fn from(graphql_status: get_user_media_list::MediaListStatus) -> Self {
        match graphql_status {
            get_user_media_list::MediaListStatus::CURRENT => MediaStatus::Current,
            get_user_media_list::MediaListStatus::PLANNING => MediaStatus::Planning,
            get_user_media_list::MediaListStatus::COMPLETED => MediaStatus::Completed,
            get_user_media_list::MediaListStatus::DROPPED => MediaStatus::Dropped,
            get_user_media_list::MediaListStatus::PAUSED => MediaStatus::Paused,
            get_user_media_list::MediaListStatus::REPEATING => MediaStatus::Repeating,
            get_user_media_list::MediaListStatus::Other(_) => MediaStatus::Unknown,
        }
    }
}
pub struct UserMediaList {
    pub page_info: PageInfo,
    pub user_id: i64,
    pub items: Option<Vec<MediaListItem>>,
}

#[derive(Clone, Debug)]
pub struct NextAiringEpisode {
    pub episode: i64,
    pub airing_at: i64,
}
#[derive(Clone, Debug)]
pub struct MediaListItem {
    pub id: i64,
    pub title: String,
    pub progress: Option<i64>,
    pub total: Option<i64>,
    pub status: Option<MediaStatus>,
    pub next_airing_episode: Option<NextAiringEpisode>,
}
#[derive(PartialEq, Clone, Copy)]
pub enum BrowseCategory {
    CategoryOne,
    CategoryTwo,
    CategoryThree,
    Search,
}
impl BrowseCategory {
    pub const ALL: [BrowseCategory; 4] = [
        BrowseCategory::CategoryOne,
        BrowseCategory::CategoryTwo,
        BrowseCategory::CategoryThree,
        BrowseCategory::Search,
    ];
    pub fn next(&self) -> Self {
        match self {
            BrowseCategory::CategoryOne => BrowseCategory::CategoryTwo,
            BrowseCategory::CategoryTwo => BrowseCategory::CategoryThree,
            BrowseCategory::CategoryThree => BrowseCategory::Search,
            BrowseCategory::Search => BrowseCategory::CategoryOne,
        }
    }
    pub fn previous(&self) -> Self {
        match self {
            BrowseCategory::CategoryOne => BrowseCategory::Search,
            BrowseCategory::CategoryTwo => BrowseCategory::CategoryOne,
            BrowseCategory::CategoryThree => BrowseCategory::CategoryTwo,
            BrowseCategory::Search => BrowseCategory::CategoryThree,
        }
    }
}
impl BrowseCategory {
    pub fn to_string_user_anime(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Watching",
            BrowseCategory::CategoryTwo => "Watched",
            BrowseCategory::CategoryThree => "Planning",
            BrowseCategory::Search => "All",
        }
    }
    pub fn to_string_user_manga(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Reading",
            BrowseCategory::CategoryTwo => "Read",
            BrowseCategory::CategoryThree => "Planning",
            BrowseCategory::Search => "All",
        }
    }

    pub fn to_string_browse_anime(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Trending",
            BrowseCategory::CategoryTwo => "This Season",
            BrowseCategory::CategoryThree => "Next Season",
            BrowseCategory::Search => "Search",
        }
    }

    pub fn to_string_browse_manga(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Trending",
            BrowseCategory::CategoryTwo => "All Time Popular",
            BrowseCategory::CategoryThree => "Top Manga",
            BrowseCategory::Search => "Search",
        }
    }
}

pub struct BrowseState {
    pub loaded_view: CurrentView,
    pub media: Option<UserMediaList>,
    pub state: TableState,
    pub current_category: BrowseCategory,
}

#[derive(Clone, Copy)]
pub enum MediaType {
    Anime,
    Manga,
}
pub enum Season {
    WINTER,
    SPRING,
    SUMMER,
    FALL,
}
impl Season {
    pub const ALL: [Season; 4] = [Season::WINTER, Season::SPRING, Season::SUMMER, Season::FALL];
    pub fn next(&self) -> Self {
        match self {
            Season::WINTER => Season::SPRING,
            Season::SPRING => Season::SUMMER,
            Season::SUMMER => Season::FALL,
            Season::FALL => Season::WINTER,
        }
    }
    pub fn to_get_media_media_season(&self) -> get_media::MediaSeason {
        match self {
            Season::WINTER => get_media::MediaSeason::WINTER,
            Season::SPRING => get_media::MediaSeason::SPRING,
            Season::SUMMER => get_media::MediaSeason::SUMMER,
            Season::FALL => get_media::MediaSeason::FALL,
        }
    }
}
