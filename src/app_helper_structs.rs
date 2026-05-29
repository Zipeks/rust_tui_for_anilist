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
            CurrentView::UserAnime => "Your Anime",
            CurrentView::UserManga => "Your Manga",
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
pub enum UserMediaStatus {
    Current,
    Planning,
    Completed,
    Repeating,
    Dropped,
    Paused,
    Unknown,
}
impl UserMediaStatus {
    pub const ALL: [UserMediaStatus; 6] = [
        UserMediaStatus::Planning,
        UserMediaStatus::Current,
        UserMediaStatus::Completed,
        UserMediaStatus::Dropped,
        UserMediaStatus::Paused,
        UserMediaStatus::Repeating,
    ];
    pub fn next(&self) -> Self {
        let index = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn previous(&self) -> Self {
        let index = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn to_update_entry_status(&self) -> update_entry::MediaListStatus {
        match self {
            UserMediaStatus::Current => update_entry::MediaListStatus::CURRENT,
            UserMediaStatus::Planning => update_entry::MediaListStatus::PLANNING,
            UserMediaStatus::Completed => update_entry::MediaListStatus::COMPLETED,
            UserMediaStatus::Dropped => update_entry::MediaListStatus::DROPPED,
            UserMediaStatus::Paused => update_entry::MediaListStatus::PAUSED,
            UserMediaStatus::Repeating => update_entry::MediaListStatus::REPEATING,
            UserMediaStatus::Unknown => update_entry::MediaListStatus::Other("".to_string()),
        }
    }
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
    get_user_media_list::{self},
    update_entry,
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
impl std::fmt::Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MediaStatus::Finished => "Finished",
            MediaStatus::Releasing => "Releasing",
            MediaStatus::NotYetReleased => "Not yet released",
            MediaStatus::Cancelled => "Cancelled",
            MediaStatus::Hiatus => "Hiatus",
            MediaStatus::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

pub struct UserMediaList {
    pub page_info: PageInfo,
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
                                time_until_airing: airing.time_until_airing,
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
                            time_until_airing: airing.time_until_airing,
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
            items: Some(items),
        }
    }
}
#[derive(Clone, Debug)]
pub struct NextAiringEpisode {
    pub episode: i64,
    pub time_until_airing: i64,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
impl From<get_media_details::MediaType> for MediaType {
    fn from(data: get_media_details::MediaType) -> Self {
        match data {
            get_media_details::MediaType::ANIME => MediaType::Anime,
            get_media_details::MediaType::MANGA => MediaType::Manga,
            get_media_details::MediaType::Other(_) => MediaType::Unknown,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Date {
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub day: Option<i64>,
}
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => format!("{:04}-{:02}-{:02}", y, m, d),
            (Some(y), Some(m), None) => format!("{:04}-{:02}-??", y, m),
            (Some(y), None, None) => format!("{}", y),
            _ => "Unknown".to_string(),
        };
        write!(f, "{}", s)
    }
}
impl Date {
    pub fn empty() -> Date {
        Date {
            year: None,
            month: None,
            day: None,
        }
    }
    pub fn to_update_entry(&self) -> update_entry::FuzzyDateInput {
        update_entry::FuzzyDateInput {
            year: self.year,
            month: self.month,
            day: self.day,
        }
    }
}

#[derive(Clone)]
pub struct UserMediaDetails {
    pub media_id: i64,
    pub progress: i64,
    pub progress_volumes: Option<i64>,
    pub repeat: i64,
    pub started_at: Date,
    pub completed_at: Date,
    pub score: f64,
    pub status: UserMediaStatus,
    pub notes: String,
}
pub struct MediaDetails {
    pub titles: Titles,
    pub description: String,
    pub average_score: i64,
    pub total: Option<i64>,
    pub volumes: Option<i64>,
    pub cover_image: String,
    pub season: Season,
    pub season_year: i64,
    pub site_url: String,
    pub media_status: MediaStatus,
    pub type_: MediaType,
    pub user_media_details: Option<UserMediaDetails>,
    pub start_date: Date,
    pub end_date: Date,
    pub is_favourite: bool,
}
impl From<get_media_details::ResponseData> for MediaDetails {
    fn from(data: get_media_details::ResponseData) -> Self {
        let media = data.media;

        let titles = if let Some(t) = media.as_ref().and_then(|x| x.title.as_ref()) {
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

        let average_score = media.as_ref().and_then(|m| m.average_score).unwrap_or(0);

        let description = media
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_else(|| "No description available.".to_string())
            .replace("<br>", "\n");

        let total = media.as_ref().and_then(|m| m.chapters.or(m.episodes));
        let volumes = media.as_ref().and_then(|m| m.volumes);

        let cover_image = media
            .as_ref()
            .and_then(|m| m.cover_image.as_ref())
            .and_then(|c| c.large.clone())
            .unwrap_or_default();

        let season = media
            .as_ref()
            .and_then(|m| m.season.clone())
            .map(Season::from)
            .unwrap_or(Season::Unknown);

        let type_ = MediaType::from(media.as_ref().map(|m| m.type_.clone()).unwrap().unwrap());
        let season_year = media.as_ref().and_then(|m| m.season_year).unwrap_or(0);

        let site_url = media
            .as_ref()
            .and_then(|m| m.site_url.clone())
            .unwrap_or_default();

        let start_date = media
            .as_ref()
            .and_then(|m| m.start_date.as_ref())
            .map(|d| Date {
                year: d.year,
                month: d.month,
                day: d.day,
            })
            .unwrap_or(Date::empty());

        let end_date = media
            .as_ref()
            .and_then(|m| m.end_date.as_ref())
            .map(|d| Date {
                year: d.year,
                month: d.month,
                day: d.day,
            })
            .unwrap_or(Date::empty());

        let is_favourite = media.as_ref().map(|m| m.is_favourite).unwrap_or(false);

        let media_status = media
            .as_ref()
            .and_then(|m| m.status.clone())
            .map(MediaStatus::from)
            .unwrap_or(MediaStatus::Unknown);

        let mut user_media_details = None;
        if let Some(m) = media.as_ref().and_then(|m| m.media_list_entry.as_ref()) {
            let media_id = m.media_id;
            let score = m.score.unwrap_or(0.0);
            let progress = m.progress.unwrap_or(0);
            let status = m
                .status
                .clone()
                .map(UserMediaStatus::from)
                .unwrap_or(UserMediaStatus::Unknown);
            let progress_volumes = m.progress_volumes;
            let repeat = m.repeat.unwrap_or(0);

            let started_at = m
                .started_at
                .as_ref()
                .map(|d| Date {
                    year: d.year,
                    month: d.month,
                    day: d.day,
                })
                .unwrap_or(Date::empty());

            let completed_at = m
                .completed_at
                .as_ref()
                .map(|d| Date {
                    year: d.year,
                    month: d.month,
                    day: d.day,
                })
                .unwrap_or(Date::empty());

            let notes = m.notes.clone().unwrap_or(String::new());

            user_media_details = Some(UserMediaDetails {
                media_id,
                score,
                progress,
                progress_volumes,
                status,
                repeat,
                started_at,
                completed_at,
                notes,
            });
        }

        MediaDetails {
            titles,
            description,
            average_score,
            total,
            volumes,
            type_,
            cover_image,
            season,
            season_year,
            site_url,
            media_status,
            user_media_details,
            start_date,
            end_date,
            is_favourite,
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

impl std::fmt::Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Season::WINTER => "Winter",
            Season::SPRING => "Spring",
            Season::SUMMER => "Summer",
            Season::FALL => "Fall",
            Season::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}
#[derive(Clone, Copy)]
pub enum CurrentEditField {
    Status,
    Score,
    EpisodeProgress,
    VolumeProgress,
    Rewatch,
    StartDate,
    EndDate,
    Notes,
}

pub enum ActivePopup {
    TitleLanguage,
    Error,
    EditMedia,
}
