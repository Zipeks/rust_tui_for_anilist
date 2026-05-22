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

use moka::ops::compute::Op;
use ratatui::widgets::TableState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserMediaStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
    Unknown,
}
impl UserMediaStatus {
    pub const ALL: [UserMediaStatus; 6] = [
        UserMediaStatus::Current,
        UserMediaStatus::Planning,
        UserMediaStatus::Completed,
        UserMediaStatus::Dropped,
        UserMediaStatus::Paused,
        UserMediaStatus::Repeating,
    ];
}

impl std::fmt::Display for UserMediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UserMediaStatus::Current => "Current",
            UserMediaStatus::Planning => "Planning",
            UserMediaStatus::Completed => "Completed",
            UserMediaStatus::Dropped => "Dropped",
            UserMediaStatus::Paused => "Paused",
            UserMediaStatus::Repeating => "Repeating",
            UserMediaStatus::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}
use crate::anilist::{
    get_media, get_media_details,
    get_user_media_list::{self, MediaListStatus},
};
impl From<get_user_media_list::MediaListStatus> for UserMediaStatus {
    fn from(graphql_status: get_user_media_list::MediaListStatus) -> Self {
        match graphql_status {
            get_user_media_list::MediaListStatus::CURRENT => UserMediaStatus::Current,
            get_user_media_list::MediaListStatus::PLANNING => UserMediaStatus::Planning,
            get_user_media_list::MediaListStatus::COMPLETED => UserMediaStatus::Completed,
            get_user_media_list::MediaListStatus::DROPPED => UserMediaStatus::Dropped,
            get_user_media_list::MediaListStatus::PAUSED => UserMediaStatus::Paused,
            get_user_media_list::MediaListStatus::REPEATING => UserMediaStatus::Repeating,
            get_user_media_list::MediaListStatus::Other(_) => UserMediaStatus::Unknown,
        }
    }
}
impl From<get_media_details::MediaListStatus> for UserMediaStatus {
    fn from(graphql_status: get_media_details::MediaListStatus) -> Self {
        match graphql_status {
            get_media_details::MediaListStatus::CURRENT => UserMediaStatus::Current,
            get_media_details::MediaListStatus::PLANNING => UserMediaStatus::Planning,
            get_media_details::MediaListStatus::COMPLETED => UserMediaStatus::Completed,
            get_media_details::MediaListStatus::DROPPED => UserMediaStatus::Dropped,
            get_media_details::MediaListStatus::PAUSED => UserMediaStatus::Paused,
            get_media_details::MediaListStatus::REPEATING => UserMediaStatus::Repeating,
            get_media_details::MediaListStatus::Other(_) => UserMediaStatus::Unknown,
        }
    }
}

pub enum MediaStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
    Unknown,
}

impl From<get_media_details::MediaStatus> for MediaStatus {
    fn from(graphql_status: get_media_details::MediaStatus) -> Self {
        match graphql_status {
            get_media_details::MediaStatus::FINISHED => MediaStatus::Finished,
            get_media_details::MediaStatus::RELEASING => MediaStatus::Releasing,
            get_media_details::MediaStatus::NOT_YET_RELEASED => MediaStatus::NotYetReleased,
            get_media_details::MediaStatus::CANCELLED => MediaStatus::Cancelled,
            get_media_details::MediaStatus::HIATUS => MediaStatus::Hiatus,
            get_media_details::MediaStatus::Other(_) => MediaStatus::Unknown,
        }
    }
}

pub struct UserMediaList {
    pub page_info: PageInfo,
    pub user_id: i64,
    pub items: Option<Vec<MediaListItem>>,
}
impl From<get_media::ResponseData> for UserMediaList {
    fn from(data: get_media::ResponseData) -> Self {
        let mut page_info = PageInfo {
            current_page: 1,
            per_page: 50,
            total: None,
            last_page: None,
            has_next_page: None,
        };
        let mut items = Vec::new();

        if let Some(page) = data.page {
            if let Some(pi) = page.page_info {
                page_info.current_page = pi.current_page.unwrap_or(1);
                page_info.per_page = pi.per_page.unwrap_or(50);
                page_info.total = pi.total;
                page_info.last_page = pi.last_page;
                page_info.has_next_page = pi.has_next_page;
            }

            if let Some(media_array) = page.media {
                for m in media_array.into_iter().flatten() {
                    let id = m.id;

                    let titles = if let Some(t) = m.title.as_ref() {
                        Titles {
                            user_preferred: t
                                .user_preferred
                                .clone()
                                .unwrap_or_else(|| "Unknown".to_string()),
                            romaji: t.romaji.clone().unwrap_or_default(),
                            english: t.english.clone().unwrap_or_default(),
                            native: t.native.clone().unwrap_or_default(),
                        }
                    } else {
                        Titles {
                            user_preferred: "Unknown".to_string(),
                            romaji: "".to_string(),
                            english: "".to_string(),
                            native: "".to_string(),
                        }
                    };

                    let type_ = m.type_.map(MediaType::from).unwrap_or(MediaType::Unknown);
                    let total = m.episodes.or(m.chapters);
                    let next_episode =
                        m.next_airing_episode
                            .as_ref()
                            .map(|airing| NextAiringEpisode {
                                airing_at: airing.airing_at,
                                episode: airing.episode,
                            });
                    let average_score = m.average_score;
                    items.push(MediaListItem {
                        id,
                        titles,
                        progress: None,
                        total,
                        type_,
                        average_score,
                        status: None,
                        next_airing_episode: next_episode,
                    });
                }
            }
        }

        UserMediaList {
            page_info,
            user_id: 0,
            items: Some(items),
        }
    }
}
impl From<get_user_media_list::ResponseData> for UserMediaList {
    fn from(data: get_user_media_list::ResponseData) -> Self {
        let mut page_info = PageInfo {
            current_page: 1,
            per_page: 50,
            total: None,
            last_page: None,
            has_next_page: None,
        };
        let mut items = Vec::new();

        if let Some(page) = data.page {
            if let Some(pi) = page.page_info {
                page_info.current_page = pi.current_page.unwrap_or(1);
                page_info.per_page = pi.per_page.unwrap_or(50);
                page_info.total = pi.total;
                page_info.last_page = pi.last_page;
                page_info.has_next_page = pi.has_next_page;
            }

            if let Some(media_list) = page.media_list {
                for m in media_list.into_iter().flatten() {
                    let id = m.media.as_ref().map(|x| x.id).unwrap_or(0);
                    let titles = if let Some(t) = m.media.as_ref().and_then(|x| x.title.as_ref()) {
                        Titles {
                            user_preferred: t
                                .user_preferred
                                .clone()
                                .unwrap_or_else(|| "Unknown".to_string()),
                            romaji: t.romaji.clone().unwrap_or_default(),
                            english: t.english.clone().unwrap_or_default(),
                            native: t.native.clone().unwrap_or_default(),
                        }
                    } else {
                        Titles {
                            user_preferred: "Unknown".to_string(),
                            romaji: "".to_string(),
                            english: "".to_string(),
                            native: "".to_string(),
                        }
                    };

                    let mut total = None;
                    let type_ = m
                        .media
                        .as_ref()
                        .and_then(|med| med.type_.clone())
                        .map(MediaType::from)
                        .unwrap_or(MediaType::Unknown);

                    if let Some(m) = &m.media {
                        total = m.episodes.or(m.chapters);
                    };

                    let next_episode = m
                        .media
                        .as_ref()
                        .and_then(|next| next.next_airing_episode.clone())
                        .map(|airing| NextAiringEpisode {
                            airing_at: airing.airing_at,
                            episode: airing.episode,
                        });
                    let mapped_status: Option<UserMediaStatus> = m.status.map(|s| s.into());
                    items.push(MediaListItem {
                        id,
                        titles,
                        progress: m.progress,
                        total,
                        type_,
                        average_score: None,
                        status: mapped_status,
                        next_airing_episode: next_episode,
                    });
                }
            }
        }

        UserMediaList {
            page_info,
            user_id: 0,
            items: Some(items),
        }
    }
}
#[derive(Clone, Debug)]
pub struct NextAiringEpisode {
    pub episode: i64,
    pub airing_at: i64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleLanguage {
    UserPreferred,
    Romaji,
    English,
    Native,
}

impl TitleLanguage {
    pub const ALL: [TitleLanguage; 4] = [
        TitleLanguage::UserPreferred,
        TitleLanguage::Romaji,
        TitleLanguage::English,
        TitleLanguage::Native,
    ];

    pub fn to_string(&self) -> &'static str {
        match self {
            TitleLanguage::UserPreferred => "User Preferred",
            TitleLanguage::Romaji => "Romaji",
            TitleLanguage::English => "English",
            TitleLanguage::Native => "Native (Kanji)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Titles {
    pub user_preferred: String,
    pub romaji: String,
    pub english: String,
    pub native: String,
}

impl Titles {
    pub fn get_title(&self, language: &TitleLanguage) -> &str {
        match language {
            TitleLanguage::UserPreferred => &self.user_preferred,
            TitleLanguage::Romaji => {
                if !self.romaji.is_empty() {
                    &self.romaji
                } else {
                    &self.user_preferred
                }
            }
            TitleLanguage::English => {
                if !self.english.is_empty() {
                    &self.english
                } else {
                    &self.romaji
                }
            }
            TitleLanguage::Native => {
                if !self.native.is_empty() {
                    &self.native
                } else {
                    &self.romaji
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct MediaListItem {
    pub id: i64,
    pub titles: Titles,
    pub progress: Option<i64>,
    pub total: Option<i64>,
    pub status: Option<UserMediaStatus>,
    pub average_score: Option<i64>,
    pub next_airing_episode: Option<NextAiringEpisode>,
    pub type_: MediaType,
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

#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Anime,
    Manga,
    Unknown,
}
impl MediaType {
    pub fn to_get_media_details(&self) -> get_media_details::MediaType {
        match self {
            MediaType::Anime => get_media_details::MediaType::ANIME,
            MediaType::Manga => get_media_details::MediaType::MANGA,
            MediaType::Unknown => get_media_details::MediaType::Other("".to_string()),
        }
    }
}
impl From<get_media::MediaType> for MediaType {
    fn from(data: get_media::MediaType) -> Self {
        match data {
            get_media::MediaType::ANIME => MediaType::Anime,
            get_media::MediaType::MANGA => MediaType::Manga,
            get_media::MediaType::Other(_) => MediaType::Unknown,
        }
    }
}
impl From<get_user_media_list::MediaType> for MediaType {
    fn from(data: get_user_media_list::MediaType) -> Self {
        match data {
            get_user_media_list::MediaType::ANIME => MediaType::Anime,
            get_user_media_list::MediaType::MANGA => MediaType::Manga,
            get_user_media_list::MediaType::Other(_) => MediaType::Unknown,
        }
    }
}
pub struct UserMediaDetails {
    pub progress: i64,
    pub score: f64,
    pub status: UserMediaStatus,
}
pub struct MediaDetails {
    pub title: String,
    pub description: String,
    pub average_score: i64,
    pub total: Option<i64>,
    pub cover_image: String,
    pub season: Season,
    pub season_year: i64,
    pub site_url: String,
    pub media_status: MediaStatus,
    pub user_media_details: Option<UserMediaDetails>,
}
impl From<get_media_details::ResponseData> for MediaDetails {
    fn from(data: get_media_details::ResponseData) -> Self {
        let media = data.media;

        let title = media
            .as_ref()
            .and_then(|m| m.title.as_ref())
            .and_then(|t| t.user_preferred.clone())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let average_score = media.as_ref().and_then(|m| m.average_score).unwrap_or(0);

        let description = media
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_else(|| "No description available.".to_string());

        let total = media.as_ref().and_then(|m| m.chapters.or(m.episodes));

        let cover_image = media
            .as_ref()
            .and_then(|m| m.cover_image.as_ref())
            .and_then(|c| c.medium.clone())
            .unwrap_or_default();

        let season = media
            .as_ref()
            .and_then(|m| m.season.clone())
            .map(Season::from)
            .unwrap_or(Season::Unknown);

        let season_year = media.as_ref().and_then(|m| m.season_year).unwrap_or(0);

        let site_url = media
            .as_ref()
            .and_then(|m| m.site_url.clone())
            .unwrap_or_default();

        let media_status = media
            .as_ref()
            .and_then(|m| m.status.clone())
            .map(MediaStatus::from)
            .unwrap_or(MediaStatus::Unknown);

        let mut user_media_details = None;
        if let Some(m) = media.as_ref().and_then(|m| m.media_list_entry.as_ref()) {
            user_media_details = Some(UserMediaDetails {
                score: m.score.unwrap_or(0.0),
                progress: m.progress.unwrap_or(0),
                status: m
                    .status
                    .clone()
                    .map(UserMediaStatus::from)
                    .unwrap_or(UserMediaStatus::Unknown),
            });
        }

        MediaDetails {
            title,
            description,
            average_score,
            total,
            cover_image,
            season,
            season_year,
            site_url,
            media_status,
            user_media_details,
        }
    }
}
pub enum Season {
    WINTER,
    SPRING,
    SUMMER,
    FALL,
    Unknown,
}
impl Season {
    pub const ALL: [Season; 4] = [Season::WINTER, Season::SPRING, Season::SUMMER, Season::FALL];
    pub fn next(&self) -> Self {
        match self {
            Season::WINTER => Season::SPRING,
            Season::SPRING => Season::SUMMER,
            Season::SUMMER => Season::FALL,
            Season::FALL => Season::WINTER,
            Season::Unknown => Season::Unknown,
        }
    }
    pub fn to_get_media_media_season(&self) -> get_media::MediaSeason {
        match self {
            Season::WINTER => get_media::MediaSeason::WINTER,
            Season::SPRING => get_media::MediaSeason::SPRING,
            Season::SUMMER => get_media::MediaSeason::SUMMER,
            Season::FALL => get_media::MediaSeason::FALL,
            Season::Unknown => get_media::MediaSeason::Other("".to_string()),
        }
    }
}
impl From<get_media_details::MediaSeason> for Season {
    fn from(value: get_media_details::MediaSeason) -> Self {
        match value {
            get_media_details::MediaSeason::WINTER => Season::WINTER,
            get_media_details::MediaSeason::SPRING => Season::SPRING,
            get_media_details::MediaSeason::SUMMER => Season::SUMMER,
            get_media_details::MediaSeason::FALL => Season::FALL,
            _ => Season::Unknown,
        }
    }
}
