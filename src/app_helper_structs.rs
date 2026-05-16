#[derive(PartialEq)]
pub enum ActiveBlock {
    Sidebar,
    Center,
    Details,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CurrentView {
    Home,
    BrowseAnime,
    BrowseManga,
    Profile,
}

impl CurrentView {
    pub const ALL: [CurrentView; 4] = [
        CurrentView::Home,
        CurrentView::BrowseAnime,
        CurrentView::BrowseManga,
        CurrentView::Profile,
    ];
}
impl std::fmt::Display for CurrentView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CurrentView::Home => "Home",
            CurrentView::BrowseAnime => "Browse Anime",
            CurrentView::BrowseManga => "Browse Manga",
            CurrentView::Profile => "Profile",
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
#[derive(Clone, Copy, PartialEq)]
pub enum MediaTab {
    Anime,
    Manga,
}

impl MediaTab {
    pub fn next(&self) -> Self {
        match self {
            MediaTab::Anime => MediaTab::Manga,
            MediaTab::Manga => MediaTab::Anime,
        }
    }

    pub fn previous(&self) -> Self {
        self.next()
    }
}
pub struct PageInfo {
    pub current_page: i64,
    pub per_page: i64,
    pub total: Option<i64>,
    pub last_page: Option<i64>,
    pub has_next_page: Option<bool>,
}
use std::fmt::write;

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
use crate::anilist::get_user_media_list::{self, MediaListStatus};
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
    pub type_: MediaType,
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
    SearchResults,
}
impl BrowseCategory {
    pub const ALL: [BrowseCategory; 4] = [
        BrowseCategory::CategoryOne,
        BrowseCategory::CategoryTwo,
        BrowseCategory::CategoryThree,
        BrowseCategory::SearchResults,
    ];
    pub fn next(&self) -> Self {
        match self {
            BrowseCategory::CategoryOne => BrowseCategory::CategoryTwo,
            BrowseCategory::CategoryTwo => BrowseCategory::CategoryThree,
            BrowseCategory::CategoryThree => BrowseCategory::SearchResults,
            BrowseCategory::SearchResults => BrowseCategory::CategoryOne,
        }
    }
    pub fn previous(&self) -> Self {
        match self {
            BrowseCategory::CategoryOne => BrowseCategory::SearchResults,
            BrowseCategory::CategoryTwo => BrowseCategory::CategoryOne,
            BrowseCategory::CategoryThree => BrowseCategory::CategoryTwo,
            BrowseCategory::SearchResults => BrowseCategory::CategoryThree,
        }
    }
}
impl BrowseCategory {
    pub fn to_string_anime(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Trending",
            BrowseCategory::CategoryTwo => "This Season",
            BrowseCategory::CategoryThree => "Next Season",
            BrowseCategory::SearchResults => "Search",
        }
    }

    pub fn to_string_manga(&self) -> &'static str {
        match self {
            BrowseCategory::CategoryOne => "Trending",
            BrowseCategory::CategoryTwo => "All Time Popular",
            BrowseCategory::CategoryThree => "Top Manga",
            BrowseCategory::SearchResults => "Search",
        }
    }
}

pub struct BrowseState {
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
}
